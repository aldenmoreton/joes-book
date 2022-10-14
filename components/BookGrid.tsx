import BookCard from './BookCard';
import Grid from '@mui/material/Grid';


type CardInfo = {
	name: string,
    owner: string,
    id: string,
    img: string
}
type props = {
	cards: Array<CardInfo>
}
export default function BookGrid({ cards }: props) {
	return (
		<Grid container spacing={1} justifyContent='center'>
			{cards.map((card: CardInfo, idx: number) => {
				return (
					<Grid item key={idx.toString()}>
						<BookCard data={card}></BookCard>
					</Grid>
				)
			})}
		</Grid>
	)
}
