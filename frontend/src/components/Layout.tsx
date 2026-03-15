import { Outlet } from 'react-router'
import PromptList from '../pages/PromptList'

export default function Layout() {
  return (
    <div className="flex h-screen">
      <aside className="w-64 border-r border-gray-200 bg-gray-50 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <h1 className="text-lg font-bold">System Prompts</h1>
        </div>
        <div className="flex-1 overflow-y-auto">
          <PromptList />
        </div>
      </aside>

      <main className="flex-1 overflow-y-auto">
        <Outlet />
      </main>
    </div>
  )
}
