import { privacyPolicy } from '../content/legal.ts'

/** Bump together with the documents in content/legal.ts; users are asked again when it changes. */
export const TERMS_VERSION = privacyPolicy.version

/**
 * Phase-one servers are outside mainland China, which needs a separate consent (PRD 11). Build with
 * VITE_DATA_OVERSEAS=false once the service has moved to mainland servers.
 */
export const DATA_OVERSEAS = (import.meta.env.VITE_DATA_OVERSEAS as string | undefined) !== 'false'
