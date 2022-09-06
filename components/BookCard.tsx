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
import { CardActionArea } from '@mui/material';

export default function BookCard() {

	return (
		<Card raised sx={{ maxWidth:'350px' }}>
			<CardActionArea>
				<CardContent>
					<CardHeader
						title='2022 Pick Six'
						subheader='Created by Joe Tosney'
					/>
				</CardContent>
			</CardActionArea>
		</Card>
	)
}
