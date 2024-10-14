<script lang="ts">
    import { Page, Navbar, Block, Popup, View } from "framework7-svelte";
    import type { NChat, EnrichedContact } from "../types/nostr";
    import Avatar from "../components/Avatar.svelte";
    import Name from "../components/Name.svelte";
    import { npubFromPubkey } from "../utils/nostr";
    let {
        chat,
        pubkey,
        enrichedContact,
    }: { chat: NChat; pubkey: string; enrichedContact: EnrichedContact } = $props();
</script>

<Popup push>
    <View tab tabActive id="chat-info-popup-view">
        <Page class="group-info-page bg-gray-900">
            <Navbar class="group-info-navbar justify-start py-8">
                <div slot="title" class="title-profile-link flex flex-row gap-2 items-center">
                    <div class="flex flex-col">Info</div>
                </div>
            </Navbar>
            <Block strong inset outline>
                <div class="flex flex-col justify-start items-center gap-4">
                    <Avatar {pubkey} picture={enrichedContact?.metadata?.picture} pxSize={128} />

                    <h1 class="text-3xl font-bold">
                        <Name
                            {pubkey}
                            metadata={enrichedContact?.metadata}
                            extraClasses="!text-3xl !font-bold"
                        />
                    </h1>
                    <p class="text-gray-500 font-light flex flex-row gap-2 items-center">
                        {npubFromPubkey(pubkey)}
                    </p>
                    <p class="px-8">{enrichedContact?.metadata?.about}</p>
                    <p class="flex flex-row items-center justify-center gap-4">
                        <span>{enrichedContact?.metadata?.nip05}</span>
                        <span class="text-gray-500 font-black">⋅</span>
                        <span>{enrichedContact?.metadata?.website}</span>
                    </p>
                </div>
            </Block>
        </Page>
    </View>
</Popup>
