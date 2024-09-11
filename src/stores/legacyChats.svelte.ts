import {get} from 'svelte/store'
import type { NDKEvent } from "@nostr-dev-kit/ndk";
import {currentIdentity, nip04Decrypt} from './identities'
import {ndkStore} from './ndk'

export type NDKEventWithPlaintext = NDKEvent & {
  plaintext: string
}

export const getLegacyChats = () => {
  const pubkey = get(currentIdentity)
  let events = $state<NDKEventWithPlaintext[]>([])

  let sub

  if (pubkey) {
    const filters = [
      {kinds: [4], authors: [pubkey]},
      {kinds: [4], '#p': [pubkey]},
    ]

    sub = ndkStore.subscribe(filters)

    sub.on('event', async event => {
      const recipient = event.pubkey === pubkey
        ? event.tags.find(t => t[0] === "p")?.[1]!
        : event.pubkey

      try {
        events = [
          ...events,
          Object.assign(event, {
            plaintext: await nip04Decrypt(recipient, event.content),
          }) as NDKEventWithPlaintext,
        ]
      } catch (e) {
        console.warn(e)
      }
    })
  }

  return {
    get events() { return events },
    get conversations() { return new Set(events.map(e => e.pubkey)) },
    sub,
  }
}
