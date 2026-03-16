import { Routes, Route, Navigate, Link } from 'react-router'
import Layout from './components/Layout'
import ErrorBoundary from './components/ErrorBoundary'
import ProjectEditor from './pages/ProjectEditor'
import ProviderSettings from './pages/ProviderSettings'

function NotFound() {
  return (
    <div className="flex flex-col items-center justify-center h-full text-gray-400">
      <h2 className="text-2xl font-semibold mb-2">404 - Page Not Found</h2>
      <Link to="/projects" className="text-blue-500 hover:underline">
        Go to Projects
      </Link>
    </div>
  )
}

export default function App() {
  return (
    <ErrorBoundary>
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<Navigate to="/projects" replace />} />
          <Route path="/projects" element={
            <div className="flex items-center justify-center h-full text-gray-400">
              Select a project or create a new one
            </div>
          } />
          <Route path="/projects/:id" element={<ProjectEditor />} />
          <Route path="/settings/providers" element={<ProviderSettings />} />
          <Route path="*" element={<NotFound />} />
        </Route>
      </Routes>
    </ErrorBoundary>
  )
}
