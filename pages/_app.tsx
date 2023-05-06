import { SessionProvider } from "next-auth/react"

import Frame from "../components/Frame"

import '../styles/globals.css'
import { ThemeProvider } from "@emotion/react"
import theme from '../lib/theme'
import { Session } from "inspector"

//TODO: Type definitions
export default function App({ Component, pageProps }: any) {
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
