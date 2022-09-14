import NextAuth, { NextAuthOptions } from "next-auth"
import { MongoDBAdapter } from "@next-auth/mongodb-adapter"
import clientPromise from "../../../lib/mongodb"
import FacebookProvider from "next-auth/providers/facebook"
import GithubProvider from "next-auth/providers/github"
import GoogleProvider from "next-auth/providers/google"
import { Session } from "inspector"

//TODO: Implement Auth with different providers
//TODO: Add custom sign in page?
//TODO: Add interface for session
interface SessionProps {
  session: any,
  user: any
}
export const authOptions: NextAuthOptions = {
  secret: process.env.NEXTAUTH_SECRET,
  adapter: MongoDBAdapter(clientPromise),
  providers: [
    // FacebookProvider({
    //   clientId: process.env.FACEBOOK_ID,
    //   clientSecret: process.env.FACEBOOK_SECRET,
    // }),
    GithubProvider({
      clientId: process.env.GITHUB_ID!,
      clientSecret: process.env.GITHUB_SECRET!,
    }),
    // GoogleProvider({
    //   clientId: process.env.GOOGLE_ID,
    //   clientSecret: process.env.GOOGLE_SECRET,
    // }),
  ],
  theme: {
    colorScheme: "light",
  },
  callbacks: {
    async session({ session, user }: SessionProps) {
      if (session?.user) {
        session.user.id = user.id;
        session.user.books = user.books;
      }
      return session;
    },
    async jwt({ token }) {
      token.userRole = "admin"
      return token
    },
  },
  session: { strategy: "database"}
}

export default NextAuth(authOptions)