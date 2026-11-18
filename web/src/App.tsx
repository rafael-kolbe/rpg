import { LoginPage } from './account/LoginPage.tsx'
import { RecoverPage } from './account/RecoverPage.tsx'
import { SignUpPage } from './account/SignUpPage.tsx'
import { HomePage } from './pages/HomePage.tsx'
import { SandboxPage } from './sandbox/SandboxPage.tsx'

/** Routes by path. Links are plain `<a href>`: each page loads fresh. */
export function App() {
  switch (window.location.pathname.replace(/\/$/, '')) {
    case '/signup':
      return <SignUpPage />
    case '/login':
      return <LoginPage />
    case '/recover':
      return <RecoverPage />
  }
  const sandbox = /^\/sandbox\/([a-z0-9-]+)\/?$/.exec(window.location.pathname)
  if (sandbox?.[1]) return <SandboxPage scenario={sandbox[1]} />
  return <HomePage />
}
