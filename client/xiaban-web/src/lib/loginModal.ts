import { createContext, useContext } from 'react'

export interface LoginModalControl {
  /** Opens the login dialog over the current page; the session starts first if a public page had skipped it. */
  openLogin: () => void
}

export const LoginModalContext = createContext<LoginModalControl | null>(null)

export function useLoginModal(): LoginModalControl {
  const value = useContext(LoginModalContext)
  if (!value) throw new Error('useLoginModal must be used inside LoginModalContext')
  return value
}
