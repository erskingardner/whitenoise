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
  - ✅ Allow for inviting OS contacts to download the app
  - ✅ Allow for inviting Nostr contacts to download the app via NIP-17 or NIP-04
  - search should also search all of nostr for a user by npub, nprofile, name, or hex pubkey (NIP-50 + Primal cache)
  - filter contacts by active conversations
  - sort contacts by last message, last seen, or name
  - show conversation transcripts on click in the main panel (how does this work with both legacy and MLS chats?)
- NIP-04
- NIP-17
- NIP-104 (MLS)
  - ✅ Create & publish prekey events
    - Adding someone manually with prekey event in person
  - ✅ Create 1:1 DM group using prekeys
  - ✅ Create & Publish welcome message
  - ✅ Create nostr group struct and store it locally in the database
  - ✅ Handle receiving welcome message from MLS
  - ✅ Show conversations list
  - ✅ Show conversation transcript
  - ✅ Send & receive messages
  - ✅ Parse nostr events in messages
    - ✅ Kind: 9
    - Reactions
    - Replies (as quotes?)
    - Media
  - ✅ Show conversation details when tapping on conversation header
  - ✅ More complete relay handling
- ✅ Simple 1-2 step onboarding flow
  - ✅ Help users set up 10051 relay list, create first key packages, etc.
- Link to help docs (where?)
- Once logged in and AppSettings are loaded, start the main app
  - ✅ Fetch profile for user
  - ✅ Fetch contacts for user
  - ✅ Fetch NIP-04 DMs for user
  - Fetch NIP-17 DMs for user
  - ✅ Fetch Prekey events for user
- Make sure that when current identity changes, we update the UI
  - ✅ Contacts
  - ✅ Chats
  - ✅ Legacy chats
- Mobile
  - ✅ Need to make sure the view collapses to mobile size properly

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
- ✅ Contacts are showing the wrong user's when two user's are logged in.
- ✅ Need to scope storage to each user better
  - ✅ Groups
  - ✅ Invites

## 🆘 Errors

- ✅ Handle errors
  ✅ - We use `thiserror` to create custom errors for our main methods and propogate errors to commands as strings.
  ✅ - Errors from tauri commands propogate back to the UI layer as string errors. We show them where needed with toasts.

## 🔐 Security 

- CSP in tuari.conf.json - Needs to be locked down as much as possible
- Isolation mode?

## 💬 MLS

- Grease values in prekeys, groups, etc.
- ⛔ WONT DO for now: Custom Nostr credential - can we create a Credential with out own type? 


## 📱 Mobile

### Android

- ✅ Nav bars need to have the hamburger removed 
- ✅ Contact + button needs to be on the same level as the title

### iOS

- Need to be able to build to phone

## Refactoring

- ✅ Update nostr_client methods to query db and check relays and not throw an error if we're offline.
- ✅ Need to improve how we handle relays in general
- ✅ Replace all mentions of prekey with key package

## Updates to match the NIP

- Need to verify that incoming messages pubkey and identity key match.
- Need to rotate signing keys on entrance into the group. 
- Need to check admin_pubkeys when processing proposals and commits. 
- 