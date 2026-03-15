use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use futures::StreamExt;
use sqlx::SqlitePool;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::crypto;
use crate::llm::{self, LlmRequest, StreamEvent};
use crate::models::{AiModel, Provider, Run, RunRequest, RunResult, RunWithResult};

pub async fn create_run(
    State(pool): State<SqlitePool>,
    Path(test_case_id): Path<String>,
    Json(payload): Json<RunRequest>,
) -> Result<(StatusCode, Json<Vec<Run>>), StatusCode> {
    // Fetch test case
    let test_case = sqlx::query_as::<_, crate::models::TestCase>(
        "SELECT * FROM test_cases WHERE id = ?",
    )
    .bind(&test_case_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let mut runs = Vec::new();

    // Fetch all prospective layers for the project once
    let all_layers = sqlx::query_as::<_, crate::models::PromptLayer>(
        "SELECT * FROM prompt_layers WHERE project_id = ?",
    )
    .bind(&test_case.project_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for model_id in &payload.model_ids {
        // Fetch model and provider info for this specific run
        let model = sqlx::query_as::<_, AiModel>("SELECT * FROM models WHERE id = ?")
            .bind(model_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Filter and merge layers for THIS model
        let mut filtered_layers: Vec<&crate::models::PromptLayer> = all_layers
            .iter()
            .filter(|l| {
                match l.layer_type.as_str() {
                    "global" | "project" => true,
                    "provider" => l.target_ref == model.provider_id,
                    "model" => l.target_ref == model.id,
                    _ => false,
                }
            })
            .collect();
        
        filtered_layers.sort_by_key(|l| match l.layer_type.as_str() {
            "global" => 1,
            "project" => 2,
            "provider" => 3,
            "model" => 4,
            _ => 5,
        });

        let mut system_prompt = filtered_layers
            .iter()
            .filter(|l| !l.content.trim().is_empty())
            .map(|l| l.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        // Apply variables
        if !payload.variables.is_empty() {
            let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
            system_prompt = re
                .replace_all(&system_prompt, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    payload.variables
                        .get(var_name)
                        .cloned()
                        .unwrap_or_else(|| format!("{{{{{}}}}}", var_name))
                })
                .to_string();
        }

        let id = ulid::Ulid::new().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO runs (id, test_case_id, model_id, status, system_prompt, started_at, created_at)
             VALUES (?, ?, ?, 'running', ?, ?, ?)",
        )
        .bind(&id)
        .bind(&test_case_id)
        .bind(model_id)
        .bind(&system_prompt)
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let run = Run {
            id: id.clone(),
            test_case_id: test_case_id.clone(),
            model_id: model_id.clone(),
            status: "running".to_string(),
            system_prompt: system_prompt.clone(),
            started_at: Some(now.clone()),
            finished_at: None,
            created_at: now,
        };
        runs.push(run.clone());

        // Spawn async task to execute the LLM call
        let pool_clone = pool.clone();
        let user_message = test_case.user_message.clone();
        let run_clone = run;

        tokio::spawn(async move {
            execute_run(pool_clone, run_clone, user_message).await;
        });
    }

    Ok((StatusCode::CREATED, Json(runs)))
}

async fn execute_run(pool: SqlitePool, run: Run, user_message: String) {
    // Fetch model and provider
    let model = match sqlx::query_as::<_, AiModel>("SELECT * FROM models WHERE id = ?")
        .bind(&run.model_id)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(m)) => m,
        _ => {
            let _ = save_error(&pool, &run.id, "Model not found").await;
            return;
        }
    };

    let provider = match sqlx::query_as::<_, Provider>("SELECT * FROM providers WHERE id = ?")
        .bind(&model.provider_id)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(p)) => p,
        _ => {
            let _ = save_error(&pool, &run.id, "Provider not found").await;
            return;
        }
    };

    let api_key = match crypto::decrypt(&provider.encrypted_api_key) {
        Ok(k) => k,
        Err(e) => {
            let _ = save_error(&pool, &run.id, &format!("Key decryption error: {}", e)).await;
            return;
        }
    };

    let llm_provider = llm::create_provider(&provider.api_type, &provider.base_url, &api_key);

    let request = LlmRequest {
        system_prompt: run.system_prompt.clone(),
        user_message,
        model_name: model.model_name.clone(),
        max_tokens: None,
        temperature: None,
    };

    let start = std::time::Instant::now();

    match llm_provider.complete(&request).await {
        Ok(response) => {
            let latency = start.elapsed().as_millis() as i64;
            let now = chrono::Utc::now().to_rfc3339();
            let result_id = ulid::Ulid::new().to_string();
            let token_json =
                serde_json::to_string(&response.token_usage).unwrap_or_else(|_| "{}".to_string());

            let _ = sqlx::query(
                "INSERT INTO run_results (id, run_id, response_text, token_usage, latency_ms, raw_response, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&result_id)
            .bind(&run.id)
            .bind(&response.content)
            .bind(&token_json)
            .bind(latency)
            .bind(&response.raw_response)
            .bind(&now)
            .execute(&pool)
            .await;

            let _ = sqlx::query(
                "UPDATE runs SET status = 'completed', finished_at = ? WHERE id = ?",
            )
            .bind(&now)
            .bind(&run.id)
            .execute(&pool)
            .await;
        }
        Err(_) => {
            let now = chrono::Utc::now().to_rfc3339();
            let _ = sqlx::query("UPDATE runs SET status = 'failed', finished_at = ? WHERE id = ?")
                .bind(&now)
                .bind(&run.id)
                .execute(&pool)
                .await;
        }
    }

    // Evaluate assertions
    let assertions = match sqlx::query_as::<_, crate::models::Assertion>(
        "SELECT * FROM assertions WHERE test_case_id = ?",
    )
    .bind(&run.test_case_id)
    .fetch_all(&pool)
    .await
    {
        Ok(a) => a,
        Err(_) => {
            let _ = generate_report(&pool, &run.id).await;
            return;
        }
    };

    // We need the response to evaluate assertions. Fetch the latest result for this run.
    let run_result = match sqlx::query_as::<_, crate::models::RunResult>(
        "SELECT * FROM run_results WHERE run_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&run.id)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => r,
        _ => {
            let _ = generate_report(&pool, &run.id).await;
            return;
        }
    };

    // Reconstruct LlmResponse for evaluator
    let llm_res = crate::llm::LlmResponse {
        content: run_result.response_text,
        token_usage: serde_json::from_str(&run_result.token_usage).unwrap_or(crate::llm::TokenUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }),
        raw_response: run_result.raw_response,
    };

    for assertion in assertions {
        if let Some(evaluator) =
            crate::llm::assertions::create_evaluator(&assertion.assertion_type, &assertion.config)
        {
            let (passed, evidence) = evaluator.evaluate(&llm_res);
            let result_id = ulid::Ulid::new().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            let _ = sqlx::query(
                "INSERT INTO assertion_results (id, run_id, assertion_id, passed, evidence, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&result_id)
            .bind(&run.id)
            .bind(&assertion.id)
            .bind(if passed { 1 } else { 0 })
            .bind(evidence)
            .bind(&now)
            .execute(&pool)
            .await;
        }
    }

    // Generate report
    let _ = generate_report(&pool, &run.id).await;
}

async fn generate_report(pool: &SqlitePool, run_id: &str) -> Result<(), StatusCode> {
    let run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = ?")
        .bind(run_id)
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = sqlx::query_as::<_, RunResult>("SELECT * FROM run_results WHERE run_id = ? ORDER BY created_at DESC LIMIT 1")
        .bind(run_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let assertions_results = sqlx::query("SELECT passed FROM assertion_results WHERE run_id = ?")
        .bind(run_id)
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut passed_count = 0;
    let mut failed_count = 0;
    for row in assertions_results {
        let passed: bool = sqlx::Row::get(&row, 0);
        if passed { passed_count += 1; } else { failed_count += 1; }
    }

    let (latency, token_usage, total_tokens) = if let Some(r) = result {
        let usage: crate::llm::TokenUsage = serde_json::from_str(&r.token_usage).unwrap_or(crate::llm::TokenUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });
        (r.latency_ms.unwrap_or(0), usage, usage.total_tokens)
    } else {
        (0, crate::llm::TokenUsage { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 }, 0)
    };

    // Calculate cost
    let pricing = sqlx::query_as::<_, crate::models::ModelPricing>("SELECT * FROM model_pricing WHERE model_id = ?")
        .bind(&run.model_id)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

    let cost = if let Some(p) = pricing {
        (token_usage.prompt_tokens as f64 / 1000.0 * p.input_1k_tokens_usd) +
        (token_usage.completion_tokens as f64 / 1000.0 * p.output_1k_tokens_usd)
    } else {
        0.0
    };

    let report_id = ulid::Ulid::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO run_reports (id, run_id, total_latency_ms, total_tokens, estimated_cost_usd, assertion_passed_count, assertion_failed_count, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&report_id)
    .bind(run_id)
    .bind(latency)
    .bind(total_tokens as i64)
    .bind(cost)
    .bind(passed_count)
    .bind(failed_count)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

async fn save_error(pool: &SqlitePool, run_id: &str, error: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    let result_id = ulid::Ulid::new().to_string();

    let _ = sqlx::query(
        "INSERT INTO run_results (id, run_id, response_text, token_usage, latency_ms, raw_response, error_message, created_at)
         VALUES (?, ?, '', '{}', NULL, '', ?, ?)",
    )
    .bind(&result_id)
    .bind(run_id)
    .bind(error)
    .bind(&now)
    .execute(pool)
    .await;

    let _ = sqlx::query("UPDATE runs SET status = 'failed', finished_at = ? WHERE id = ?")
        .bind(&now)
        .bind(run_id)
        .execute(pool)
        .await;
}

pub async fn list_runs(
    State(pool): State<SqlitePool>,
    Path(test_case_id): Path<String>,
) -> Result<Json<Vec<RunWithResult>>, StatusCode> {
    let runs = sqlx::query_as::<_, Run>(
        "SELECT * FROM runs WHERE test_case_id = ? ORDER BY created_at DESC",
    )
    .bind(&test_case_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut results = Vec::new();
    for run in runs {
        let result = sqlx::query_as::<_, RunResult>(
            "SELECT * FROM run_results WHERE run_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(&run.id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        results.push(RunWithResult { run, result });
    }

    Ok(Json(results))
}

pub async fn get_run(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<RunWithResult>, StatusCode> {
    let run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let result = sqlx::query_as::<_, RunResult>(
        "SELECT * FROM run_results WHERE run_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&run.id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RunWithResult { run, result }))
}

pub async fn stream_run(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Fetch model and provider
    let model = sqlx::query_as::<_, AiModel>("SELECT * FROM models WHERE id = ?")
        .bind(&run.model_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let provider = sqlx::query_as::<_, Provider>("SELECT * FROM providers WHERE id = ?")
        .bind(&model.provider_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let api_key = crypto::decrypt(&provider.encrypted_api_key)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let llm_provider = llm::create_provider(&provider.api_type, &provider.base_url, &api_key);

    // Fetch test case for user_message
    let test_case = sqlx::query_as::<_, crate::models::TestCase>(
        "SELECT * FROM test_cases WHERE id = ?",
    )
    .bind(&run.test_case_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let request = LlmRequest {
        system_prompt: run.system_prompt.clone(),
        user_message: test_case.user_message,
        model_name: model.model_name,
        max_tokens: None,
        temperature: None,
    };

    let (tx, rx) = mpsc::channel::<StreamEvent>(100);

    tokio::spawn(async move {
        if let Err(e) = llm_provider.stream(&request, tx.clone()).await {
            let _ = tx
                .send(StreamEvent {
                    event_type: "error".to_string(),
                    content: None,
                    token_usage: None,
                    error: Some(e),
                })
                .await;
        }
    });

    let stream = ReceiverStream::new(rx).map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
        Ok(Event::default().data(data))
    });

    Ok(Sse::new(stream))
}
