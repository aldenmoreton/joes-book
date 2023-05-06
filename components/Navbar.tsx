import * as React from 'react'
import AppBar from '@mui/material/AppBar'
import Container from '@mui/system/Container'
import Toolbar from '@mui/material/Toolbar'
import AdbIcon from '@mui/icons-material/Adb'
import Typography from '@mui/material/Typography'
import Box from '@mui/material/Box'
import IconButton from '@mui/material/IconButton'
import MenuIcon from '@mui/icons-material/Menu'
import Menu from '@mui/material/Menu'
import MenuItem from '@mui/material/MenuItem'
import Button from '@mui/material/Button'
import Tooltip from '@mui/material/Tooltip'
import Avatar from '@mui/material/Avatar'
import { signOut } from 'next-auth/react'
import Link from '@mui/material/Link'

//TODO: Implement everything; pages, logos, etc
interface props {
	img: string | undefined
}
export default function Navbar({ img }: props) {

	const [anchorElNav, setAnchorElNav] = React.useState<null | HTMLElement>(null);
  	const [anchorElUser, setAnchorElUser] = React.useState<null | HTMLElement>(null);

	const handleOpenNavMenu = (event: React.MouseEvent<HTMLElement>) => {
		setAnchorElNav(event.currentTarget);
	};
  const handleOpenUserMenu = (event: React.MouseEvent<HTMLElement>) => {
	setAnchorElUser(event.currentTarget);
  };

  const handleCloseNavMenu = () => {
	setAnchorElNav(null);
  };

  const handleCloseUserMenu = () => {
	setAnchorElUser(null);
  };


  return (
	<>
		<AppBar position="static" sx={{mb: 3}}>
		<Container maxWidth="xl">
			<Toolbar disableGutters>
			<AdbIcon sx={{ display: { xs: 'none', md: 'flex' }, mr: 1 }} />
			<Typography
				variant="h6"
				noWrap
				component="a"
				href="/"
				sx={{
				mr: 2,
				display: { xs: 'none', md: 'flex' },
				fontFamily: 'monospace',
				fontWeight: 700,
				letterSpacing: '.3rem',
				color: 'inherit',
				textDecoration: 'none',
				}}
			>
				JOE'S BOOK
			</Typography>

			<Box sx={{ flexGrow: 1, display: { xs: 'flex', md: 'none' } }}>
				<IconButton
				size="large"
				aria-label="account of current user"
				aria-controls="menu-appbar"
				aria-haspopup="true"
				onClick={handleOpenNavMenu}
				color="inherit"
				>
				<MenuIcon />
				</IconButton>
				<Menu
				id="menu-appbar"
				anchorEl={anchorElNav}
				anchorOrigin={{
					vertical: 'bottom',
					horizontal: 'left',
				}}
				keepMounted
				transformOrigin={{
					vertical: 'top',
					horizontal: 'left',
				}}
				open={Boolean(anchorElNav)}
				onClose={handleCloseNavMenu}
				sx={{
					display: { xs: 'block', md: 'none' },
				}}
				>
				<MenuItem key={'books'}>
					<Link href='/books' textAlign="center" underline='none' color='black'>books</Link>
				</MenuItem>
				</Menu>
			</Box>
			<AdbIcon sx={{ display: { xs: 'flex', md: 'none' }, mr: 1 }} />
			<Typography
				variant="h5"
				noWrap
				component="a"
				href="/"
				sx={{
				mr: 2,
				display: { xs: 'flex', md: 'none' },
				flexGrow: 1,
				fontFamily: 'monospace',
				fontWeight: 700,
				letterSpacing: '.3rem',
				color: 'inherit',
				textDecoration: 'none',
				}}
			>
				JOE'S BOOK
			</Typography>
			<Box sx={{ flexGrow: 1, display: { xs: 'none', md: 'flex' } }}>
				<Button
					key={'books'}
					href='/books'
					sx={{ my: 2, color: 'white', display: 'block' }}
				>
				Books
				</Button>
			</Box>

			<Box sx={{ flexGrow: 0 }}>
				<Tooltip title="Open settings">
				<IconButton onClick={handleOpenUserMenu} sx={{ p: 0 }}>
					<Avatar alt="Remy Sharp" src={img} />
				</IconButton>
				</Tooltip>
				<Menu
				sx={{ mt: '45px' }}
				id="menu-appbar"
				anchorEl={anchorElUser}
				anchorOrigin={{
					vertical: 'top',
					horizontal: 'right',
				}}
				keepMounted
				transformOrigin={{
					vertical: 'top',
					horizontal: 'right',
				}}
				open={Boolean(anchorElUser)}
				onClose={handleCloseUserMenu}
				>
				<MenuItem key={'books'} onClick={() => signOut()}>
					<Typography textAlign="center">books</Typography>
				</MenuItem>
				</Menu>
			</Box>
			</Toolbar>
		</Container>
		</AppBar>
	</>
  );

}
