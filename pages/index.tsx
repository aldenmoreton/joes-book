import Head from 'next/head'
import React from 'react'
import Button from '@mui/material/Button'
import { useSession, signOut } from 'next-auth/react'

export default function Home({ }) {
	const { data: session } = useSession()

	return (
		<>
		{/* <div className="container"> */}
		<Head>
			{/* <title>Create Next App</title> */}
			<link rel="icon" href="/favicon.ico"/>
		</Head>
		<main>
			<h1 className="">
				Welcome to Joe's Book {session!.user!.name}
			</h1>
			<Button variant='outlined' href='/books'>Go to Books!</Button>
			<button onClick={() => signOut()}></button>
		</main>
		{/* </div> */}
		</>
	)
}
