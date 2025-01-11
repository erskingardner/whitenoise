# White Noise Messaging

## ✨ App Features

- Chat details screen with members list, media, chat settings, etc.
- Reactions
- Replies (as quotes?)
- Disappearing messages
- Remote signing
- Duress PIN codes (one deletes all the messages in all chats, another deletes ALL data)
- Allow for inviting Nostr contacts to download the app via NIP-17 or NIP-04
- search should also search all of nostr for a user by npub, nprofile, name, or hex pubkey (NIP-50 + Primal cache)
- Adding another device to conversation flow
- filter chats by unread conversations
- filter contacts by active conversations
- sort contacts by last message, last seen, or name
- Adding someone manually with prekey event in person
- Media (blossom?)
- Allow for dismissing unprocessable invites and store what we've seen/dismissed and don't show them in the UI again.

## 📑 Marketing website (https://whitenoise.chat)

- Logo
- Simple, clean, no-nonsense design
- Documentation & FAQ
- Long form articles? (Blog)

## 🔐 Security

- CSP in tuari.conf.json - Needs to be locked down as much as possible
- Isolation mode?
- Use OS keyring as much as possible

## 💬 MLS Library

- Grease values in prekeys, groups, etc.

## 🦄 Updates to match the NIP

- Need to verify that incoming messages pubkey and identity key match.
- Need to rotate signing keys on entrance into the group.
- Need to check admin_pubkeys when processing proposals and commits.
- Need to regularly clean up old key packages from MLS storage.
- Need to cache exporter secrets (use a config for how long to keep them around)
