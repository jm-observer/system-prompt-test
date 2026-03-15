import { Routes, Route, Navigate } from 'react-router'
import Layout from './components/Layout'
import PromptEditor from './pages/PromptEditor'

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<Navigate to="/prompts" replace />} />
        <Route path="/prompts" element={<PromptEditor />} />
        <Route path="/prompts/:id" element={<PromptEditor />} />
      </Route>
    </Routes>
  )
}
