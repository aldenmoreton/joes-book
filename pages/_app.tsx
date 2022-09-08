import { SessionProvider } from "next-auth/react"

import type { AppProps } from "next/app"
import Navbar from "../components/Navbar"

import '../styles/globals.css'

// Use of the <SessionProvider> is mandatory to allow components that call
// `useSession()` anywhere in your application to access the `session` object.
export default function App({ Component, pageProps }: AppProps) {
  return (
    <SessionProvider session={pageProps.session} refetchInterval={0}>
        <Navbar></Navbar>
          <Component {...pageProps} />
    </SessionProvider>
  )
}