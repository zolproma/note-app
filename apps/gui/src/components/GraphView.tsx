import { useEffect, useState, useRef, useCallback } from "react";
import { invoke, type GraphData, type GraphNode, type GraphEdge } from "../tauri";
import { useI18n } from "../i18n";

interface GraphViewProps {
  onOpenNote: (id: string) => void;
}

interface SimNode extends GraphNode {
  x: number;
  y: number;
  vx: number;
  vy: number;
}

function GraphView({ onOpenNote }: GraphViewProps) {
  const t = useI18n();
  const [data, setData] = useState<GraphData | null>(null);
  const [loading, setLoading] = useState(true);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const nodesRef = useRef<SimNode[]>([]);
  const edgesRef = useRef<GraphEdge[]>([]);
  const animRef = useRef<number>(0);
  const dragRef = useRef<{ node: SimNode | null; offsetX: number; offsetY: number }>({ node: null, offsetX: 0, offsetY: 0 });
  const hoveredRef = useRef<string | null>(null);

  useEffect(() => {
    invoke<GraphData>("get_graph_data")
      .then(setData)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  // Initialize simulation
  useEffect(() => {
    if (!data || !canvasRef.current) return;
    const canvas = canvasRef.current;
    const w = canvas.parentElement?.clientWidth || 800;
    const h = canvas.parentElement?.clientHeight || 600;
    canvas.width = w;
    canvas.height = h;

    // Initialize node positions in a circle
    const cx = w / 2;
    const cy = h / 2;
    const r = Math.min(w, h) * 0.35;
    nodesRef.current = data.nodes.map((n, i) => ({
      ...n,
      x: cx + r * Math.cos((2 * Math.PI * i) / data.nodes.length),
      y: cy + r * Math.sin((2 * Math.PI * i) / data.nodes.length),
      vx: 0,
      vy: 0,
    }));
    edgesRef.current = data.edges;

    // Run force simulation
    let ticks = 0;
    function tick() {
      const nodes = nodesRef.current;
      const edges = edgesRef.current;
      const alpha = Math.max(0.001, 1 - ticks * 0.008);

      // Repulsion (all pairs)
      for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
          const dx = nodes[j].x - nodes[i].x;
          const dy = nodes[j].y - nodes[i].y;
          const d = Math.sqrt(dx * dx + dy * dy) || 1;
          const force = (200 * alpha) / d;
          nodes[i].vx -= (dx / d) * force;
          nodes[i].vy -= (dy / d) * force;
          nodes[j].vx += (dx / d) * force;
          nodes[j].vy += (dy / d) * force;
        }
      }

      // Attraction (edges)
      const nodeMap = new Map(nodes.map((n) => [n.id, n]));
      for (const edge of edges) {
        const src = nodeMap.get(edge.source);
        const tgt = nodeMap.get(edge.target);
        if (!src || !tgt) continue;
        const dx = tgt.x - src.x;
        const dy = tgt.y - src.y;
        const d = Math.sqrt(dx * dx + dy * dy) || 1;
        const force = (d - 120) * 0.02 * alpha;
        src.vx += (dx / d) * force;
        src.vy += (dy / d) * force;
        tgt.vx -= (dx / d) * force;
        tgt.vy -= (dy / d) * force;
      }

      // Center gravity
      for (const node of nodes) {
        node.vx += (cx - node.x) * 0.005 * alpha;
        node.vy += (cy - node.y) * 0.005 * alpha;
      }

      // Apply velocity with damping
      for (const node of nodes) {
        if (dragRef.current.node?.id === node.id) continue;
        node.vx *= 0.6;
        node.vy *= 0.6;
        node.x += node.vx;
        node.y += node.vy;
        // Bounds
        node.x = Math.max(30, Math.min(w - 30, node.x));
        node.y = Math.max(30, Math.min(h - 30, node.y));
      }

      ticks++;
      draw();
      animRef.current = requestAnimationFrame(tick);
    }

    function draw() {
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      ctx.clearRect(0, 0, w, h);

      const nodes = nodesRef.current;
      const edges = edgesRef.current;
      const nodeMap = new Map(nodes.map((n) => [n.id, n]));

      // Draw edges
      ctx.strokeStyle = "rgba(0, 0, 0, 0.08)";
      ctx.lineWidth = 1;
      for (const edge of edges) {
        const src = nodeMap.get(edge.source);
        const tgt = nodeMap.get(edge.target);
        if (!src || !tgt) continue;
        ctx.beginPath();
        ctx.moveTo(src.x, src.y);
        ctx.lineTo(tgt.x, tgt.y);
        ctx.stroke();
      }

      // Draw nodes
      for (const node of nodes) {
        const isHovered = hoveredRef.current === node.id;
        const radius = 6 + Math.min(node.link_count * 2, 10);

        // Node circle
        ctx.beginPath();
        ctx.arc(node.x, node.y, radius, 0, Math.PI * 2);
        ctx.fillStyle = isHovered
          ? "#4a7396"
          : node.lifecycle === "active"
            ? "rgba(93, 139, 179, 0.7)"
            : node.lifecycle === "inbox"
              ? "rgba(196, 160, 80, 0.7)"
              : "rgba(134, 133, 173, 0.5)";
        ctx.fill();

        // Label
        ctx.font = isHovered ? "bold 12px -apple-system, sans-serif" : "11px -apple-system, sans-serif";
        ctx.fillStyle = isHovered ? "#1d1d1f" : "#48484a";
        ctx.textAlign = "center";
        const label = node.title.length > 20 ? node.title.slice(0, 20) + "..." : node.title;
        ctx.fillText(label, node.x, node.y + radius + 14);
      }
    }

    tick();
    return () => cancelAnimationFrame(animRef.current);
  }, [data]);

  // Mouse interaction
  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;

    if (dragRef.current.node) {
      dragRef.current.node.x = mx - dragRef.current.offsetX;
      dragRef.current.node.y = my - dragRef.current.offsetY;
      dragRef.current.node.vx = 0;
      dragRef.current.node.vy = 0;
      return;
    }

    let found: string | null = null;
    for (const node of nodesRef.current) {
      const r = 6 + Math.min(node.link_count * 2, 10);
      const dx = mx - node.x;
      const dy = my - node.y;
      if (dx * dx + dy * dy < r * r) {
        found = node.id;
        break;
      }
    }
    hoveredRef.current = found;
    canvas.style.cursor = found ? "pointer" : "default";
  }, []);

  const handleMouseDown = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;
    for (const node of nodesRef.current) {
      const r = 6 + Math.min(node.link_count * 2, 10);
      const dx = mx - node.x;
      const dy = my - node.y;
      if (dx * dx + dy * dy < r * r) {
        dragRef.current = { node, offsetX: dx, offsetY: dy };
        return;
      }
    }
  }, []);

  const handleMouseUp = useCallback(() => {
    dragRef.current = { node: null, offsetX: 0, offsetY: 0 };
  }, []);

  const handleClick = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    if (dragRef.current.node) return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;
    for (const node of nodesRef.current) {
      const r = 6 + Math.min(node.link_count * 2, 10);
      const dx = mx - node.x;
      const dy = my - node.y;
      if (dx * dx + dy * dy < r * r) {
        onOpenNote(node.id);
        return;
      }
    }
  }, [onOpenNote]);

  if (loading) return <div className="empty-state"><div className="empty-state-desc">Loading...</div></div>;

  if (!data || data.nodes.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-title">{t.noNotesToGraph}</div>
        <div className="empty-state-desc">{t.noNotesToGraphDesc}</div>
      </div>
    );
  }

  return (
    <div style={{ width: "100%", height: "100%", position: "relative" }}>
      <div className="graph-info">
        {data.nodes.length} {t.graphNodes}, {data.edges.length} {t.graphEdges}. {t.graphClickOpen}
      </div>
      <canvas
        ref={canvasRef}
        onMouseMove={handleMouseMove}
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        onClick={handleClick}
        style={{ width: "100%", height: "100%" }}
      />
    </div>
  );
}

export default GraphView;
