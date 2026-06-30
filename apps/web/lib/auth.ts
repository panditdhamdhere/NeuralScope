import { betterAuth } from "better-auth";
import { drizzleAdapter } from "better-auth/adapters/drizzle";

import { db } from "@/lib/db";
import { authSchema } from "@/lib/db/schema";

export const auth = betterAuth({
  database: drizzleAdapter(db, {
    provider: "pg",
    schema: authSchema,
  }),
  secret: process.env.BETTER_AUTH_SECRET,
  baseURL: process.env.BETTER_AUTH_URL ?? process.env.NEXT_PUBLIC_APP_URL ?? "http://localhost:3000",
  emailAndPassword: {
    enabled: true,
    requireEmailVerification: process.env.APP_ENV === "production",
  },
  session: {
    expiresIn: 60 * 60 * 24 * 7,
    cookieCache: {
      enabled: true,
      maxAge: 60 * 5,
    },
  },
  advanced: {
    database: {
      // Generate UUIDs for all models (users, accounts, sessions).
      // "uuid" mode only auto-fills user rows on Postgres; account/session TEXT ids stay null.
      generateId: () => crypto.randomUUID(),
    },
    useSecureCookies: process.env.APP_ENV === "production",
  },
  trustedOrigins: [
    process.env.BETTER_AUTH_URL ?? "http://localhost:3000",
    process.env.NEXT_PUBLIC_APP_URL ?? "http://localhost:3000",
    process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080",
  ].filter((value, index, all) => all.indexOf(value) === index),
});

export type Session = typeof auth.$Infer.Session;
