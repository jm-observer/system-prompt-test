import { useQuery } from '@tanstack/react-query'
import { fetchReportsSummary, type ReportSummary } from '../api/reports'
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  BarChart,
  Bar,
  Cell
} from 'recharts'

export default function AnalyticsDashboard() {
  const { data: summaries = [], isLoading } = useQuery({
    queryKey: ['reports-summary'],
    queryFn: () => fetchReportsSummary(),
  })

  if (isLoading) return <div className="p-4 text-center">Loading analytics...</div>

  // Group by model for breakdown
  const modelStats = summaries.reduce((acc, curr) => {
    if (!acc[curr.model_name]) {
      acc[curr.model_name] = { name: curr.model_name, count: 0, totalTokens: 0, totalCost: 0, avgLatency: 0, passed: 0, totalAssertions: 0 }
    }
    const stat = acc[curr.model_name]
    stat.count++
    stat.totalTokens += curr.total_tokens
    stat.totalCost += curr.cost_usd
    stat.avgLatency = (stat.avgLatency * (stat.count - 1) + curr.latency_ms) / stat.count
    stat.passed += curr.assertions_passed
    stat.totalAssertions += (curr.assertions_passed + curr.assertions_failed)
    return acc
  }, {} as Record<string, any>)

  const modelData = Object.values(modelStats)

  // Reverse summaries for timeline (it's returned latest first)
  const timelineData = [...summaries].reverse().map((s, i) => ({
    name: `Run ${i + 1}`,
    latency: s.latency_ms,
    tokens: s.total_tokens,
    cost: s.cost_usd * 100, // scaled for visibility
    success: s.assertions_failed === 0 ? 1 : 0
  }))

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <StatCard title="Total Runs" value={summaries.length} />
        <StatCard title="Avg Latency" value={`${Math.round(modelData.reduce((a, b) => a + b.avgLatency, 0) / (modelData.length || 1))}ms`} />
        <StatCard title="Est. Total Cost" value={`$${modelData.reduce((a, b) => a + b.totalCost, 0).toFixed(4)}`} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white p-4 rounded-lg shadow-sm border">
          <h4 className="text-sm font-medium mb-4">Performance Trend (Latency)</h4>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={timelineData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="name" hide />
                <YAxis />
                <Tooltip />
                <Line type="monotone" dataKey="latency" stroke="#3b82f6" strokeWidth={2} dot={false} />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>

        <div className="bg-white p-4 rounded-lg shadow-sm border">
          <h4 className="text-sm font-medium mb-4">Cost Distribution by Model</h4>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={modelData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="name" />
                <YAxis />
                <Tooltip />
                <Bar dataKey="totalCost" fill="#10b981">
                  {modelData.map((_entry, index) => (
                    <Cell key={`cell-${index}`} fill={['#3b82f6', '#10b981', '#f59e0b', '#ef4444'][index % 4]} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>

      <div className="bg-white p-4 rounded-lg shadow-sm border overflow-x-auto">
        <h4 className="text-sm font-medium mb-4">Model Comparison</h4>
        <table className="min-w-full text-sm">
          <thead>
            <tr className="border-b text-gray-500 text-left">
              <th className="pb-2 font-medium">Model</th>
              <th className="pb-2 font-medium">Runs</th>
              <th className="pb-2 font-medium">Avg Latency</th>
              <th className="pb-2 font-medium">Pass Rate</th>
              <th className="pb-2 font-medium">Total Cost</th>
            </tr>
          </thead>
          <tbody>
            {modelData.map((stat: any) => (
              <tr key={stat.name} className="border-b last:border-0 hover:bg-gray-50">
                <td className="py-2 font-medium">{stat.name}</td>
                <td className="py-2">{stat.count}</td>
                <td className="py-2">{Math.round(stat.avgLatency)}ms</td>
                <td className="py-2">
                  <span className={`px-2 py-0.5 rounded text-xs ${stat.totalAssertions === 0 ? 'bg-gray-100' : stat.passed === stat.totalAssertions ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'}`}>
                    {stat.totalAssertions === 0 ? 'N/A' : `${((stat.passed / stat.totalAssertions) * 100).toFixed(1)}%`}
                  </span>
                </td>
                <td className="py-2">${stat.totalCost.toFixed(4)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}

function StatCard({ title, value }: { title: string, value: string | number }) {
  return (
    <div className="bg-white p-4 rounded-lg shadow-sm border">
      <p className="text-xs text-gray-500 uppercase tracking-wider">{title}</p>
      <p className="text-2xl font-bold mt-1">{value}</p>
    </div>
  )
}
