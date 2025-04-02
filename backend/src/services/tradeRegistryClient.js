import { PublicKey, SystemProgram, TransactionInstruction } from '@solana/web3.js';
import { createHash } from 'node:crypto';

/**
 * Instruction layouts mirror programs/crates/dex-core (Borsh).
 * Keep in sync with TradeRegistryInstruction enum.
 */
const COPY_ACTION = { buy: 0, sell: 1 };

const DEFAULT_PROGRAM_ID =
  process.env.TRADE_REGISTRY_PROGRAM_ID ||
  'TradeRegistry1111111111111111111111111111111';

export function getTradeRegistryProgramId() {
  return new PublicKey(DEFAULT_PROGRAM_ID);
}

export function findRegistryPda(programId = getTradeRegistryProgramId()) {
  return PublicKey.findProgramAddressSync([Buffer.from('registry')], programId);
}

export function findLeaderPda(leader, programId = getTradeRegistryProgramId()) {
  const leaderKey = new PublicKey(leader);
  return PublicKey.findProgramAddressSync(
    [Buffer.from('leader'), leaderKey.toBuffer()],
    programId,
  );
}

export function findSubscriptionPda(follower, leader, programId = getTradeRegistryProgramId()) {
  const followerKey = new PublicKey(follower);
  const leaderKey = new PublicKey(leader);
  return PublicKey.findProgramAddressSync(
    [Buffer.from('sub'), followerKey.toBuffer(), leaderKey.toBuffer()],
    programId,
  );
}

function encodeU32(value) {
  const buf = Buffer.alloc(4);
  buf.writeUInt32LE(value);
  return buf;
}

function encodeU16(value) {
  const buf = Buffer.alloc(2);
  buf.writeUInt16LE(value);
  return buf;
}

function encodeU64(value) {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(BigInt(value));
  return buf;
}

function encodeOptionU32(value) {
  if (value === undefined || value === null) return Buffer.from([0]);
  return Buffer.concat([Buffer.from([1]), encodeU32(value)]);
}

function encodeOptionBool(value) {
  if (value === undefined || value === null) return Buffer.from([0]);
  return Buffer.concat([Buffer.from([1]), Buffer.from([value ? 1 : 0])]);
}

function variantIndex(index, payload = Buffer.alloc(0)) {
  return Buffer.concat([Buffer.from([index]), payload]);
}

export function encodeInitializeRegistry() {
  return variantIndex(0);
}

export function encodeRegisterLeader(maxFollowers) {
  return variantIndex(1, encodeU32(maxFollowers));
}

export function encodeUpdateLeader({ maxFollowers, isActive } = {}) {
  return variantIndex(
    2,
    Buffer.concat([encodeOptionU32(maxFollowers), encodeOptionBool(isActive)]),
  );
}

export function encodeSubscribe(sizeBps) {
  return variantIndex(3, encodeU16(sizeBps));
}

export function encodeUnsubscribe() {
  return variantIndex(4);
}

export function encodeLogCopyIntent({ action, mint, amountLamports, referenceSig }) {
  const mintKey = new PublicKey(mint);
  const sig =
    typeof referenceSig === 'string'
      ? Buffer.from(referenceSig, 'hex')
      : Buffer.from(referenceSig || []);
  const paddedSig = Buffer.alloc(64);
  sig.copy(paddedSig, 0, 0, Math.min(sig.length, 64));

  return variantIndex(
    5,
    Buffer.concat([
      Buffer.from([COPY_ACTION[action] || COPY_ACTION.buy]),
      mintKey.toBuffer(),
      encodeU64(amountLamports),
      paddedSig,
    ]),
  );
}

export function buildLogCopyIntentInstruction({
  follower,
  leader,
  action,
  mint,
  amountLamports,
  referenceSig,
  programId = getTradeRegistryProgramId(),
}) {
  const programKey = programId instanceof PublicKey ? programId : new PublicKey(programId);
  const [registry] = findRegistryPda(programKey);
  const [leaderProfile] = findLeaderPda(leader, programKey);
  const [subscription] = findSubscriptionPda(follower, leader, programKey);
  const followerKey = new PublicKey(follower);

  const ref =
    typeof referenceSig === 'string' && referenceSig.length === 64
      ? createHash('sha256').update(referenceSig).digest()
      : referenceSig;

  return new TransactionInstruction({
    programId: programKey,
    keys: [
      { pubkey: registry, isSigner: false, isWritable: true },
      { pubkey: leaderProfile, isSigner: false, isWritable: true },
      { pubkey: subscription, isSigner: false, isWritable: true },
      { pubkey: followerKey, isSigner: true, isWritable: false },
    ],
    data: encodeLogCopyIntent({ action, mint, amountLamports, referenceSig: ref }),
  });
}

export function isTradeRegistryEnabled() {
  return process.env.TRADE_REGISTRY_ENABLED === 'true';
}
