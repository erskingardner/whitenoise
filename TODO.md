# White Noise Messaging

## ✨ App Features

- ✅ Set up initial AppState
- ✅ Load DB
  - ✅ Load AppSettings from database if available, use defaults if not.
- ✅ Simple AppSettings management methods to read/write settings to/from DB
- ✅ Create new identity keys and store them
- ✅ Login with nsec
  - ✅ Get npub and add to identities in the DB
  - ✅ Set as current identity
  - ✅ Store nsec in keyring
- ✅ Logout from current identity
  - ✅ Delete nsec and remove from identities in the DB
  - ✅ Set current identity to next identity (or null if none)
- ✅ Clear all data
  - ✅ Remove all data from DB
  - ✅ Clear AppSettings
  - ✅ Remove all secrets from keyring
- ✅ Basic settings screens
  - ✅ Profiles
  - ✅ Dev tools
  - Security
- ✅ Need simple splash screen while we're waiting for identities (don't flash login screen)
- ✅ Contacts Panel
  - ✅ Load contacts from Nostr
  - ✅ Ensure that changing profiles updates the contacts list
  - ✅ search should filter your contacts by name, npub, nprofile, or hex pubkey
  - Contacts list should group contacts by chats, nostr contacts, and OS contacts
  - Allow for inviting OS contacts to download the app
  - Allow for inviting Nostr contacts to download the app via NIP-17 or NIP-04
  - search should also search all of nostr for a user by npub, nprofile, name, or hex pubkey (NIP-50 + Primal cache)
  - filter contacts by active conversations
  - sort contacts by last message, last seen, or name
  - show conversation transcripts on click in the main panel (how does this work with both legacy and MLS chats?)
- ✅ NIP-04
- NIP-17
- NIP-104 (MLS)
  ✅ Create & publish prekey events
    - Adding someone manually with prekey event in person
  - Show conversations in sidebar
  - Show conversation transcripts in main panel
  - Show participants in conversation (where do we put conversation details?)
- Simple 1-2 step onboarding flow
- Link to help docs (where?)
- Once logged in and AppSettings are loaded, start the main app
  - ✅ Fetch profile for user
  - ✅ Fetch contacts for user
  - ✅ Fetch NIP-04 DMs for user
  - Fetch NIP-17 DMs for user
  - Fetch Prekey events for user
- Make sure that when current identity changes, we update the UI
  - ✅ Contacts
  - Chats
  - ✅ Legacy chats
- Mobile
  - Need to make sure the view collapses to mobile size properly

## 📑 Marketing website (https://whitenoise.chat)
  - Simple, clean, no-nonsense design
  - Documentation & FAQ
  - Long form articles? (Blog)
  - Need a better logo and app icon?

## 🐛 Bugs

- ✅ Logging out of the last account crashes the app
- ✅ You can somehow add blank strings to the identities Vec
- ✅ You can log in with the same nsec twice
- ✅ Login screen doesn't redirect to the app anymore

## 🆘 Errors

- Where we do handle errors? 
  - We use `anyhow` to propogate the error but at some point we have to handle/show it. I think the best option is to propogate to the UI layer via an error message or toast (and allow for a retry where possible).

## 🔐 Security 

- investigate CSP in tuari.conf.json - Needs to be locked down as much as possible
- Isolation mode?