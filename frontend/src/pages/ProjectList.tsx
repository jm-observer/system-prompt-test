import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate, useParams } from 'react-router'
import { fetchProjects, createProject } from '../api/projects'

export default function ProjectList() {
  const navigate = useNavigate()
  const { id: activeId } = useParams()
  const queryClient = useQueryClient()

  const { data: projects = [], isLoading } = useQuery({
    queryKey: ['projects'],
    queryFn: fetchProjects,
  })

  const createMutation = useMutation({
    mutationFn: () => createProject({ name: 'New Project' }),
    onSuccess: (p) => {
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      navigate(`/projects/${p.id}`)
    },
  })

  if (isLoading) return <div className="p-4 text-gray-500">Loading...</div>

  return (
    <div>
      <div className="p-2">
        <button
          onClick={() => createMutation.mutate()}
          disabled={createMutation.isPending}
          className="w-full px-3 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
        >
          + New Project
        </button>
      </div>
      <ul>
        {projects.map((p) => (
          <li key={p.id}>
            <button
              onClick={() => navigate(`/projects/${p.id}`)}
              className={`w-full text-left px-4 py-2 text-sm truncate hover:bg-gray-100 ${
                activeId === p.id ? 'bg-blue-50 text-blue-700 font-medium' : ''
              }`}
            >
              {p.name}
            </button>
          </li>
        ))}
      </ul>
    </div>
  )
}
