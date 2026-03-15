import type { PromptLayer } from '../api/layers'

const LAYER_LABELS: Record<string, string> = {
  global: 'Global',
  project: 'Project',
  provider: 'Provider',
  model: 'Model',
}

interface Props {
  layers: PromptLayer[]
  activeLayerId: string | null
  onSelect: (layer: PromptLayer) => void
}

export default function LayerTabs({ layers, activeLayerId, onSelect }: Props) {
  return (
    <div className="flex border-b border-gray-200">
      {layers.map((layer) => (
        <button
          key={layer.id}
          onClick={() => onSelect(layer)}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeLayerId === layer.id
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700'
          }`}
        >
          {LAYER_LABELS[layer.layer_type] || layer.layer_type}
        </button>
      ))}
    </div>
  )
}
