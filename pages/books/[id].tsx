import { InferGetServerSidePropsType } from 'next'

import { authOptions } from '../api/auth/[...nextauth]'
import { unstable_getServerSession } from "next-auth/next"

export async function getServerSideProps(context: any) {
	const session = await unstable_getServerSession(context.req, context.res, authOptions)
	if (!session) {
		return {
			redirect: {
				permanent: false,
        		destination: "/"
			}
		}
	}


	const { id } = context.query
	return {
		props: {
			id: id
		}
	}
}

export default function Home({ id }: InferGetServerSidePropsType<typeof getServerSideProps>) {
	return (
		<>
			<p>You are on the book page</p>
			{id}
		</>
	)
}