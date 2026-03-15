import { Outlet, useNavigate } from 'react-router'
import ProjectList from '../pages/ProjectList'

export default function Layout() {
  const navigate = useNavigate()

  return (
    <div className="flex h-screen">
      <aside className="w-64 border-r border-gray-200 bg-gray-50 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <h1 className="text-lg font-bold">SP Lab</h1>
        </div>
        <div className="flex-1 overflow-y-auto">
          <ProjectList />
        </div>
        <div className="border-t border-gray-200 p-2">
          <button
            onClick={() => navigate('/settings/providers')}
            className="w-full text-left px-3 py-2 text-sm text-gray-600 hover:bg-gray-100 rounded"
          >
            Settings
          </button>
        </div>
      </aside>

      <main className="flex-1 flex flex-col overflow-hidden">
        <Outlet />
      </main>
    </div>
  )
}
