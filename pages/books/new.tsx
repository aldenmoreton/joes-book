// import { InferGetServerSidePropsType } from 'next'
import Typography from '@mui/material/Typography'

//{ }: InferGetServerSidePropsType<typeof getServerSideProps>
export default function Home() {
	return (
		<>
			<Typography align='center' variant='h1'>Create new book</Typography>
		</>
	)
}