import { PointerEvent, useRef, useState } from 'react'

const allowance = 78

export function App() {
  const [expanded, setExpanded] = useState(false)
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const [dragging, setDragging] = useState(false)
  const dragStart = useRef({ x: 0, y: 0, left: 0, top: 0 })
  const moved = useRef(false)

  const handlePointerDown = (event: PointerEvent<HTMLDivElement>) => {
    event.currentTarget.setPointerCapture(event.pointerId)
    dragStart.current = { x: event.clientX, y: event.clientY, left: position.x, top: position.y }
    moved.current = false
    setDragging(true)
  }

  const handlePointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (!dragging) return
    const nextX = dragStart.current.left + event.clientX - dragStart.current.x
    const nextY = dragStart.current.top + event.clientY - dragStart.current.y
    if (Math.abs(event.clientX - dragStart.current.x) > 3 || Math.abs(event.clientY - dragStart.current.y) > 3) {
      moved.current = true
    }
    setPosition({ x: nextX, y: nextY })
  }

  const handlePointerUp = () => {
    setDragging(false)
    if (!moved.current) setExpanded((value) => !value)
  }

  return (
    <main className="stage">
      <div
        className={`widget ${expanded ? 'widget--expanded' : ''} ${dragging ? 'widget--dragging' : ''}`}
        style={{ transform: `translate3d(${position.x}px, ${position.y}px, 0)` }}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
      >
        <div className="halo" aria-hidden="true" />
        <div className="ball" aria-label={`5 小时额度剩余 ${allowance}%`}>
          <svg className="progress" viewBox="0 0 100 100" aria-hidden="true">
            <circle className="progress__track" cx="50" cy="50" r="43" />
            <circle className="progress__value" cx="50" cy="50" r="43" pathLength="100" strokeDasharray={`${allowance} 100`} />
          </svg>
          <div className="ball__content">
            <span className="ball__number">{allowance}</span>
            <span className="ball__unit">%</span>
          </div>
          <span className="ball__label">5H</span>
        </div>
        {expanded && (
          <section className="details" onPointerDown={(event) => event.stopPropagation()}>
            <div className="details__header">
              <span>5 小时额度</span>
              <span className="status"><i />正常</span>
            </div>
            <div className="details__value">{allowance}<small>% 剩余</small></div>
            <div className="details__meta">
              <span>下次重置</span>
              <strong>2 小时 41 分</strong>
            </div>
            <div className="details__footer">演示数据 · 等待 Codex 连接</div>
          </section>
        )}
      </div>
      <p className="hint">拖动悬浮球移动位置 · 点击查看详情</p>
    </main>
  )
}