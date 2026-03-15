import { Routes, Route, Navigate } from 'react-router'
import Layout from './components/Layout'
import ProjectEditor from './pages/ProjectEditor'
import ProviderSettings from './pages/ProviderSettings'

export default function App() {
  return (
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
      </Route>
    </Routes>
  )
}
