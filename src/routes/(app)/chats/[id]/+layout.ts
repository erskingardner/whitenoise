import type { LayoutLoad } from "./$types";
import { invoke } from "@tauri-apps/api/core";
import { type NostrMlsGroup } from "$lib/types/nostr";
import { error } from "@sveltejs/kit";

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
