import { PointerEvent, useRef, useState } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

const allowance = 78

export function App() {
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const [dragging, setDragging] = useState(false)
  const dragStart = useRef({ x: 0, y: 0, left: 0, top: 0 })
  const nativeDragStarted = useRef(false)

  const handlePointerDown = (event: PointerEvent<HTMLDivElement>) => {
    event.currentTarget.setPointerCapture(event.pointerId)
    dragStart.current = { x: event.clientX, y: event.clientY, left: position.x, top: position.y }
    setDragging(true)
    if (isTauri()) {
      nativeDragStarted.current = true
      void getCurrentWindow().startDragging()
    }
  }

  const handlePointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (!dragging || isTauri()) return
    const nextX = dragStart.current.left + event.clientX - dragStart.current.x
    const nextY = dragStart.current.top + event.clientY - dragStart.current.y
    setPosition({ x: nextX, y: nextY })
  }

  const handlePointerUp = () => {
    setDragging(false)
    nativeDragStarted.current = false
  }

  return (
    <main className="stage">
      <div
        className={`widget ${dragging ? 'widget--dragging' : ''}`}
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
      </div>
    </main>
  )
}