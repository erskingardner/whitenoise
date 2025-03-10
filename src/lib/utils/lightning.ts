import type { NEvent } from '$lib/types/nostr';
import type { LightningInvoice, LightningPayment } from '$lib/types/chat';
import { findBolt11Tag, findPreimage } from './tags';
import { toDataURL } from "qrcode";

export function eventToLightningInvoice(event: NEvent): LightningInvoice | undefined {
  const bolt11Tag = findBolt11Tag(event);
  if (!bolt11Tag?.[1]) return;
  const invoice = bolt11Tag[1];
  const amount = Number(bolt11Tag[2] || 0) / 1000;
  const description = bolt11Tag[3];
  const lightningInvoice: LightningInvoice = { invoice, amount, description, isPaid: false };
  return lightningInvoice;
}

export function eventToLightningPayment(event: NEvent): LightningPayment | undefined {
  const preimage = findPreimage(event);
  if (!preimage) return;
  return { preimage, isPaid: false };
}

export async function lightningInvoiceToQRCode(invoice: string): Promise<string> {
  try {
      return await toDataURL(`lightning:${invoice}`);
  } catch (error) {
      console.error("Error generating QR code:", error);
      return "";
  }
}
