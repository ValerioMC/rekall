import type { GraphEdge, GraphNode, NodeKind, SemanticGraph } from '@/model/diagram'

export function node(id: string, kind: NodeKind, title = id, file: string | null = null): GraphNode {
  return {
    id,
    kind,
    title,
    description: null,
    sources: file ? [{ file, startLine: 1, endLine: 10, symbol: null }] : [],
    provenance: 'inferred',
    confidence: 0.8,
    metadata: {}
  }
}

export function edge(from: string, to: string, relation: string, label: string | null = null): GraphEdge {
  return { id: `${from}-${relation}-${to}`, from, to, relation, label, provenance: null, confidence: null, metadata: {} }
}

/** `process_order()` holds four conceptual steps; `charge()` holds one; a store is written. */
export function orderGraph(): SemanticGraph {
  return {
    format: 'rekall.semantic-graph',
    version: 1,
    nodes: [
      node('process', 'code', 'process_order()', 'src/order.rs'),
      node('receive', 'action', 'Receive order', 'src/order.rs'),
      node('validate', 'action', 'Validate order', './src/order.rs'),
      node('paid', 'decision', 'Payment accepted?'),
      node('charge', 'code', 'charge()', 'src/pay.rs'),
      node('capture', 'action', 'Capture payment', 'src/pay.rs'),
      node('orders', 'data', 'Orders table'),
      node('done', 'state', 'Completed')
    ],
    edges: [
      edge('process', 'receive', 'contains'),
      edge('process', 'validate', 'contains'),
      edge('process', 'paid', 'contains'),
      edge('charge', 'capture', 'contains'),
      edge('receive', 'validate', 'leads_to'),
      edge('validate', 'paid', 'leads_to'),
      edge('paid', 'capture', 'conditionally_leads_to', 'card'),
      edge('paid', 'done', 'conditionally_leads_to', 'free'),
      edge('capture', 'done', 'transitions_to'),
      edge('capture', 'orders', 'writes'),
      edge('process', 'charge', 'calls')
    ]
  }
}
