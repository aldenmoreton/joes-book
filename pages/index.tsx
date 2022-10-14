import Head from 'next/head'
import React from 'react'
import Button from '@mui/material/Button'
import Typography from '@mui/material/Typography'
import { useSession, signOut } from 'next-auth/react'

export default function Home({ }) {
	const { data: session } = useSession()

	return (
		<>
		<Head>
			<title>Joe's Book</title>
			<link rel="icon" href="/favicon.ico"/>
		</Head>
		<Typography variant='h4'>Welcome to Joe's Book {session!.user!.name}</Typography>
		<Button variant='outlined' href='/books'>Go to Books!</Button>
		<Button variant='outlined' onClick={() => signOut()}>Sign out</Button>
		</>
	)
}
