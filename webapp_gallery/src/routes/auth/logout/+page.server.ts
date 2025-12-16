import { redirect, type Cookies } from '@sveltejs/kit'
import type { PageServerLoad } from '../$types'


function logout(cookies: Cookies) {
  // eat the cookie
  cookies.set('session', '', {
    path: '/',
    expires: new Date(0),
  })

  // redirect the user
  redirect(302, '/auth')
}
export const load: PageServerLoad = async ({ cookies }) => {
  logout(cookies);
}

export const actions = {
  default({ cookies }: { cookies: Cookies }) {
    logout(cookies);
  },
}
