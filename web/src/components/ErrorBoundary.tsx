import { Component, type ReactNode } from 'react'
import { RobotStage, PrimaryButton } from './ui.tsx'

/** Without this, any render error unmounts the whole app and leaves a blank screen. */
export class ErrorBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false }

  static getDerivedStateFromError() {
    return { failed: true }
  }

  componentDidCatch(error: unknown) {
    console.error(error)
  }

  render() {
    if (!this.state.failed) return this.props.children
    return (
      <main className="mx-auto flex min-h-dvh w-full max-w-[480px] flex-col items-center justify-center px-5 text-center">
        <RobotStage spec="confused" label="小伴绊了一跤" size={170} />
        <p className="mt-6 text-window/80">页面出了点问题，重新打开一下就好。</p>
        <PrimaryButton className="mt-6 max-w-60" onClick={() => location.reload()}>重新打开</PrimaryButton>
      </main>
    )
  }
}
