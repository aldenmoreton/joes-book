import * as React from 'react';
import Card from '@mui/material/Card'
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent'
import Typography from '@mui/material/Typography'
import CardActions from '@mui/material/CardActions'
import Button from '@mui/material/Button'
import IconButton, { IconButtonProps } from '@mui/material/IconButton'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import { styled } from '@mui/material/styles'
import Collapse from '@mui/material/Collapse'
import { Avatar, CardActionArea } from '@mui/material';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Badge from '@mui/material/Badge';
import MailIcon from '@mui/icons-material/Mail'

interface CardProps {
	data: {
		name: string,
		owner: string,
		id: string,
		img: string
	}
}

//TODO: Create dynamic badge
export default function BookCard({ data }: CardProps) {
	// let chip = {
	// 	label: 'Unfinished Picks',
	// 	color: 'warning'
	// }
	// if (!data.currentPicks) {
	// 	chip.label = 'Up to date'
	// 	chip.color = 'success'
	// }
	return (
		<Card raised sx={{ maxWidth:'350px' }}>
			<CardActionArea href={`/books/${data.id}`}>
				<CardContent>
					<CardHeader
						title={data.name}
						subheader={`Created by ${data.owner}`}
						action={
							<Badge badgeContent={4} color="primary">
								<MailIcon></MailIcon>
							</Badge>
						}
						avatar={
							<Avatar src={data.img}/>
						}
					/>

				</CardContent>
			</CardActionArea>
		</Card>
	)
}
