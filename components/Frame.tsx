import { useSession, signIn } from "next-auth/react"

import Navbar from "./Navbar"

import CircularProgress from "@mui/material/CircularProgress"

interface props {
	children: React.ReactNode
}
export default function Frame({ children }: props){
	const { data: session, status } = useSession()

	if (status === 'loading') {
		return (
			<>
				<CircularProgress />
			</>
		)
	}

	if (!session) {
		return (
			<>
				Not signed in <br />
				<button onClick={() => signIn()}>Sign in</button>
    		</>
		)
	}

  	let img = (session?.user?.image) ? session.user.image : undefined

	return (
		<>
			<Navbar img={img}/>
			{children}
		</>
	)
}