import React from 'react';

import Grid from "@mui/material/Grid"
import PickSixCard from './PickSixCard';

type CardInfo = {
	home: string,
	visitor: string,
	homeSpread: string
}

type props = {
	cards: Array<CardInfo>
}
export default function PickSixGrid({cards}: props) {
	const defaultPoints = {
		one: false,
		two: false,
		three: false,
		four: false,
		five: false,
		six: false
	}
	const [test1, test2] = React.useState(defaultPoints);
	return (
		<Grid container spacing={1} justifyContent='center'>
			{cards.map((card: CardInfo, idx: number) => {
				return (
					<Grid item key={idx.toString()}>
						<PickSixCard home={card.home} visitor={card.visitor} homeSpread={card.homeSpread} pointTracker={React.useState(defaultPoints)}></PickSixCard>
					</Grid>
				)
			})}
		</Grid>
	)
}