import { SessionProvider } from "next-auth/react"

import type { AppProps } from "next/app"

import Frame from "../components/Frame"

import '../styles/globals.css'
import { ThemeProvider } from "@emotion/react"
import theme from '../lib/theme'

export default function App({ Component, pageProps }: AppProps) {
  return (
    <SessionProvider session={pageProps.session} refetchInterval={0}>
        <ThemeProvider theme={theme}>
          <Frame>
            <Component {...pageProps} />
          </Frame>
        </ThemeProvider>
    </SessionProvider>
  )
}
