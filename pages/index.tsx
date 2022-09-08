import Head from 'next/head'
import clientPromise from '../lib/mongodb'
import { InferGetServerSidePropsType } from 'next'
import React from 'react'
import Button from '@mui/material/Button'
import { useSession, signIn, signOut } from 'next-auth/react'

export default function Home({ }) {
	const { data: session, status } = useSession()
	if (status === 'loading') {
		return <h1>Loading</h1>
	}
	if (!session) {
		return (
			<>
				Not signed in <br />
				<button onClick={() => signIn()}>Sign in</button>
    		</>
		)
	}
	return (
		<div className="container">
		<Head>
			<title>Create Next App</title>
			<link rel="icon" href="/favicon.ico"/>
		</Head>
		<main>
			<h1 className="">
				Welcome to Joe's Book {session!.user!.name}
			</h1>
			<Button variant='outlined' href='/books'>Go to Books!</Button>
			<button onClick={() => signOut()}></button>
		</main>

		<footer>
		</footer>

		</div>
	)
}
