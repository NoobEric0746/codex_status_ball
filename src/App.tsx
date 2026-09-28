import { useState } from 'react'
import type { CSSProperties } from 'react'

type Parameters = {
  ballSize: number
  ballInset: number
  ringInset: number
  ringWidth: number
  numberSize: number
  numberTop: number
  resetSize: number
  resetTop: number
  panelWidth: number
  panelFont: number
  panelLeft: number
}

const initialParameters: Parameters = { ballSize: 122, ballInset: 6, ringInset: 5, ringWidth: 5, numberSize: 51, numberTop: 25, resetSize: 14, resetTop: 81, panelWidth: 236, panelFont: 25, panelLeft: 0 }
const controls: { key: keyof Parameters; label: string; min: number; max: number }[] = [
  { key: 'ballSize', label: '球体尺寸', min: 90, max: 180 }, { key: 'ballInset', label: '球体边距', min: 0, max: 20 },
  { key: 'ringInset', label: '进度环边距', min: 2, max: 30 }, { key: 'ringWidth', label: '进度环线宽', min: 1, max: 12 },
  { key: 'numberSize', label: '额度字号', min: 24, max: 76 }, { key: 'numberTop', label: '额度垂直位置', min: 0, max: 70 },
  { key: 'resetSize', label: '刷新字号', min: 8, max: 30 }, { key: 'resetTop', label: '刷新垂直位置', min: 55, max: 110 },
  { key: 'panelWidth', label: '面板宽度', min: 140, max: 300 }, { key: 'panelFont', label: '面板字号', min: 12, max: 48 },
  { key: 'panelLeft', label: '面板文字左边距', min: 0, max: 50 },
]

function rustSnippet(parameters: Parameters) {
  return `const BALL_SIZE: i32 = ${parameters.ballSize};
const PANEL_WIDTH: i32 = ${parameters.panelWidth};
Ellipse: inset ${parameters.ballInset};
Progress ring: inset ${parameters.ringInset}, width ${parameters.ringWidth};
Allowance font: ${parameters.numberSize}, top ${parameters.numberTop};
Reset font: ${parameters.resetSize}, top ${parameters.resetTop};
Panel font: ${parameters.panelFont}, left padding ${parameters.panelLeft};`
}

export function App() {
  const [parameters, setParameters] = useState(initialParameters)
  const [copied, setCopied] = useState(false)
  const update = (key: keyof Parameters, value: number) => { setParameters((current) => ({ ...current, [key]: value })); setCopied(false) }
  const copyParameters = async () => { await navigator.clipboard.writeText(rustSnippet(parameters)); setCopied(true) }
  const totalWidth = parameters.ballSize + parameters.panelWidth
  const ballStyle = { width: parameters.ballSize, height: parameters.ballSize, '--ball-inset': `${parameters.ballInset}px`, '--ring-inset': `${parameters.ringInset}px`, '--ring-width': `${parameters.ringWidth}px`, '--number-size': `${parameters.numberSize}px`, '--number-top': `${parameters.numberTop}px`, '--reset-size': `${parameters.resetSize}px`, '--reset-top': `${parameters.resetTop}px` } as CSSProperties

  return <main className="debug-shell">
    <section className="preview-panel">
      <div className="eyebrow">CODEX STATUS BALL / DEBUG</div><h1>悬浮球尺寸调试</h1><p className="intro">拖动右侧滑块，实时观察球体与展开面板的关系。</p>
      <div className="preview-stage"><div className="preview-widget" style={{ width: totalWidth, height: parameters.ballSize }}>
        <div className="preview-ball" style={ballStyle}><div className="preview-ring" /><div className="preview-number">78</div><div className="preview-reset">4h 20m</div></div>
        <div className="preview-capsule" style={{ left: parameters.ballSize, width: parameters.panelWidth, height: parameters.ballSize }}><div className="preview-panel-text" style={{ left: parameters.panelLeft, fontSize: parameters.panelFont }}><strong>WEEKLY&nbsp; 57%</strong><strong>RESET IN&nbsp; 2d</strong></div></div>
      </div></div>
      <div className="preview-caption"><span />预览尺寸 {totalWidth} × {parameters.ballSize}px</div>
    </section>
    <aside className="controls-panel">
      <div className="controls-heading"><div><div className="eyebrow">LIVE CONTROLS</div><h2>调整参数</h2></div><button className="reset-button" type="button" onClick={() => setParameters(initialParameters)}>重置</button></div>
      <div className="control-list">{controls.map((control) => <label className="control" key={control.key}><span className="control-label"><span>{control.label}</span><output>{parameters[control.key]}px</output></span><input type="range" min={control.min} max={control.max} value={parameters[control.key]} onChange={(event) => update(control.key, Number(event.target.value))} /></label>)}</div>
      <button className="copy-button" type="button" onClick={copyParameters}>{copied ? '已复制 Rust 参数' : '复制 Rust 参数'}</button><pre className="parameter-output">{rustSnippet(parameters)}</pre>
    </aside>
  </main>
}
