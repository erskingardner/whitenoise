export function hexMlsGroupId(mlsGroupId: Uint8Array): string {
    return Array.from(mlsGroupId, (byte) => byte.toString(16).padStart(2, "0")).join("");
}
