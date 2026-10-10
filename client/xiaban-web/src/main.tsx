import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App.tsx'
import { ErrorBoundary } from './components/ErrorBoundary.tsx'
import './index.css'
import { layHomeUnderActivity } from './lib/activity.ts'
import { ThemeProvider } from './lib/theme.tsx'

layHomeUnderActivity(location, history)

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider><ErrorBoundary>
      <App />
    </ErrorBoundary></ThemeProvider>
  </StrictMode>,
)
