import type { LayoutLoad } from "./$types";
import { invoke } from "@tauri-apps/api/core";
import { type NostrMlsGroup, NostrMlsGroupType } from "$lib/types/nostr";
import { error } from "@sveltejs/kit";
import { accounts } from "$lib/stores/accounts";
import { nameFromMetadata } from "$lib/utils/nostr";
import type { EnrichedContact } from "$lib/types/nostr";
import { get } from "svelte/store";

export const load = (async ({ params }) => {
    const groupResponse = await invoke("get_group", {
        groupId: params.id,
    });

    if (groupResponse instanceof Error) {
        throw error(404, { message: `Group not found: ${groupResponse.message}` });
    }

    const group = groupResponse as NostrMlsGroup;

    return {
        group,
    };
}) satisfies LayoutLoad;
