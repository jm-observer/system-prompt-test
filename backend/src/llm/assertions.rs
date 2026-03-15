use crate::llm::LlmResponse;
use serde_json::Value;

pub trait AssertionEvaluator: Send + Sync {
    fn evaluate(&self, response: &LlmResponse) -> (bool, Option<String>);
}

pub struct KeywordAssertion {
    pub keyword: String,
    pub must_present: bool,
}

impl AssertionEvaluator for KeywordAssertion {
    fn evaluate(&self, response: &LlmResponse) -> (bool, Option<String>) {
        let contains = response.content.contains(&self.keyword);
        if self.must_present {
            if contains {
                (true, Some(format!("Found keyword: '{}'", self.keyword)))
            } else {
                (false, Some(format!("Keyword '{}' not found in response", self.keyword)))
            }
        } else {
            if !contains {
                (true, Some(format!("Keyword '{}' correctly absent", self.keyword)))
            } else {
                (false, Some(format!("Keyword '{}' found but should be absent", self.keyword)))
            }
        }
    }
}

pub struct ToolCallAssertion {
    pub tool_name: Option<String>,
    pub must_call: bool,
}

impl AssertionEvaluator for ToolCallAssertion {
    fn evaluate(&self, response: &LlmResponse) -> (bool, Option<String>) {
        // This is a simplified implementation. 
        // In a real scenario, we would parse response.raw_response or response.content for tool calls.
        // Assuming raw_response is JSON and might contain tool_calls
        let v: Value = serde_json::from_str(&response.raw_response).unwrap_or(Value::Null);
        
        // Very basic check - looking for "tool_calls" in the JSON
        let has_tool_calls = v.get("tool_calls").is_some() || v.get("choices").and_then(|c| c.get(0)).and_then(|m| m.get("message")).and_then(|t| t.get("tool_calls")).is_some();
        
        // Note: Actual implementation depends on provider response format
        // For now, let's just use the boolean logic
        if self.must_call {
            if has_tool_calls {
                (true, Some("Tool call detected".to_string()))
            } else {
                (false, Some("No tool call detected but one was expected".to_string()))
            }
        } else {
            if !has_tool_calls {
                (true, Some("No tool call detected as expected".to_string()))
            } else {
                (false, Some("Tool call detected but none were expected".to_string()))
            }
        }
    }
}

pub fn create_evaluator(assertion_type: &str, config_json: &str) -> Option<Box<dyn AssertionEvaluator>> {
    let config: Value = serde_json::from_str(config_json).ok()?;
    
    match assertion_type {
        "keyword_present" => {
            let keyword = config.get("keyword")?.as_str()?.to_string();
            Some(Box::new(KeywordAssertion { keyword, must_present: true }))
        }
        "keyword_absent" => {
            let keyword = config.get("keyword")?.as_str()?.to_string();
            Some(Box::new(KeywordAssertion { keyword, must_present: false }))
        }
        "must_call" => {
            let tool_name = config.get("tool_name").and_then(|v| v.as_str()).map(|s| s.to_string());
            Some(Box::new(ToolCallAssertion { tool_name, must_call: true }))
        }
        "must_not_call" => {
            let tool_name = config.get("tool_name").and_then(|v| v.as_str()).map(|s| s.to_string());
            Some(Box::new(ToolCallAssertion { tool_name, must_call: false }))
        }
        _ => None,
    }
}
