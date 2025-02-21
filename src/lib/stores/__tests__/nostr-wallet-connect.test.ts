import { describe, it, expect } from 'vitest';
import { validateNostrWalletConnectUri } from '../accounts';

describe('validateNostrWalletConnectUri', () => {
    it('should validate a correct NWC URI with WSS relay', () => {
        const validUri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c";
        const result = validateNostrWalletConnectUri(validUri);
        expect(result.isValid).toBe(true);
        expect(result.error).toBeUndefined();
    });

    it('should validate a correct NWC URI with WS relay', () => {
        const validUri = "nostr+walletconnect://pubkey?relay=ws://relay.example.com&secret=123";
        const result = validateNostrWalletConnectUri(validUri);
        expect(result.isValid).toBe(true);
        expect(result.error).toBeUndefined();
    });

    it('should validate URI with multiple relays', () => {
        const multiRelayUri = "nostr+walletconnect://pubkey?relay=wss://relay1.com&relay=ws://relay2.com&secret=123";
        const result = validateNostrWalletConnectUri(multiRelayUri);
        expect(result.isValid).toBe(true);
        expect(result.error).toBeUndefined();
    });

    it('should reject URI without correct prefix', () => {
        const invalidPrefixUri = "nostr://something";
        const result = validateNostrWalletConnectUri(invalidPrefixUri);
        expect(result.isValid).toBe(false);
        expect(result.error).toBe("Invalid URI format: must start with 'nostr+walletconnect://'");
    });

    it('should reject URI without relay parameter', () => {
        const noRelayUri = "nostr+walletconnect://pubkey?secret=123";
        const result = validateNostrWalletConnectUri(noRelayUri);
        expect(result.isValid).toBe(false);
        expect(result.error).toBe("Missing required 'relay' parameter");
    });

    it('should reject URI with non-WS/WSS relay', () => {
        const httpRelayUri = "nostr+walletconnect://pubkey?relay=http://relay.com&secret=123";
        const result = validateNostrWalletConnectUri(httpRelayUri);
        expect(result.isValid).toBe(false);
        expect(result.error).toBe("Relay must use either WSS or WS protocol");
    });

    it('should reject URI with mixed valid/invalid relay protocols', () => {
        const mixedRelayUri = "nostr+walletconnect://pubkey?relay=wss://relay1.com&relay=http://relay2.com&secret=123";
        const result = validateNostrWalletConnectUri(mixedRelayUri);
        expect(result.isValid).toBe(false);
        expect(result.error).toBe("Relay must use either WSS or WS protocol");
    });

    it('should reject URI with invalid relay URL format', () => {
        const invalidRelayUri = "nostr+walletconnect://pubkey?relay=not-a-url&secret=123";
        const result = validateNostrWalletConnectUri(invalidRelayUri);
        expect(result.isValid).toBe(false);
        expect(result.error).toBe("Invalid relay URL format");
    });

    it('should reject URI without secret parameter', () => {
        const noSecretUri = "nostr+walletconnect://pubkey?relay=wss://relay.com";
        const result = validateNostrWalletConnectUri(noSecretUri);
        expect(result.isValid).toBe(false);
        expect(result.error).toBe("Missing required 'secret' parameter");
    });

    it('should reject malformed URI', () => {
        const malformedUri = "not-even-a-uri";
        const result = validateNostrWalletConnectUri(malformedUri);
        expect(result.isValid).toBe(false);
        expect(result.error).toBe("Invalid URI format: must start with 'nostr+walletconnect://'");
    });

    it('should reject empty URI', () => {
        const emptyUri = "";
        const result = validateNostrWalletConnectUri(emptyUri);
        expect(result.isValid).toBe(false);
        expect(result.error).toBe("Invalid URI format: must start with 'nostr+walletconnect://'");
    });

    it('should validate URI with additional parameters', () => {
        const uriWithExtraParams = "nostr+walletconnect://pubkey?relay=wss://relay.com&secret=123&extra=param";
        const result = validateNostrWalletConnectUri(uriWithExtraParams);
        expect(result.isValid).toBe(true);
        expect(result.error).toBeUndefined();
    });

    it('should validate URI with encoded characters', () => {
        const encodedUri = "nostr+walletconnect://pubkey?relay=wss%3A%2F%2Frelay.com&secret=123";
        const result = validateNostrWalletConnectUri(encodedUri);
        expect(result.isValid).toBe(true);
        expect(result.error).toBeUndefined();
    });
}); 
