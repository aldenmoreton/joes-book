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

import { useSession } from 'next-auth/react'

const pages = ['Products', 'Pricing', 'Blog'];
const settings = ['Profile', 'Account', 'Dashboard', 'Logout'];

//TODO: Implement everything; pages, logos, etc
//TODO: Interface for props
export default function Navbar( ) {

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
  const { data: session } = useSession()
  let img = (session?.user?.image) ? session.user.image : undefined
  return (
	<>
		<AppBar position="static">
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
				{pages.map((page) => (
					<MenuItem key={page} onClick={handleCloseNavMenu}>
					<Typography textAlign="center">{page}</Typography>
					</MenuItem>
				))}
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
				{pages.map((page) => (
				<Button
					key={page}
					onClick={handleCloseNavMenu}
					sx={{ my: 2, color: 'white', display: 'block' }}
				>
					{page}
				</Button>
				))}
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
				{settings.map((setting) => (
					<MenuItem key={setting} onClick={handleCloseUserMenu}>
					<Typography textAlign="center">{setting}</Typography>
					</MenuItem>
				))}
				</Menu>
			</Box>
			</Toolbar>
		</Container>
		</AppBar>
	</>
  );

}
