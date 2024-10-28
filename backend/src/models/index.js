import mongoose from 'mongoose';

const userSchema = new mongoose.Schema(
  {
    email: { type: String, unique: true, required: true },
    passwordHash: { type: String, required: true },
    name: String,
  },
  { timestamps: true },
);

const tradeSchema = new mongoose.Schema(
  {
    userId: String,
    walletId: String,
    walletPublicKey: String,
    mint: String,
    symbol: String,
    side: { type: String, enum: ['buy', 'sell'] },
    amountSol: Number,
    tokenAmount: Number,
    priceUsd: Number,
    mode: String,
    source: { type: String, enum: ['manual', 'copy', 'auto'] },
    leaderWallet: String,
    txSignature: String,
    status: String,
    idempotencyKey: { type: String, index: true, sparse: true },
  },
  { timestamps: true },
);

const walletSchema = new mongoose.Schema(
  {
    userId: String,
    publicKey: String,
    label: String,
    encryptedSecret: String,
    isPlatform: { type: Boolean, default: false },
    isExternal: { type: Boolean, default: false },
  },
  { timestamps: true },
);

const copySubSchema = new mongoose.Schema(
  {
    _id: String,
    userId: String,
    leaderAddress: String,
    followerWalletId: String,
    followerPublicKey: String,
    sizePct: { type: Number, default: 100 },
    active: { type: Boolean, default: true },
    lastSignature: String,
  },
  { timestamps: true, _id: false },
);

const autoRuleSchema = new mongoose.Schema(
  {
    _id: String,
    userId: String,
    walletId: String,
    mint: String,
    minSentiment: Number,
    maxTradeSol: Number,
    active: { type: Boolean, default: true },
  },
  { timestamps: true, _id: false },
);

const sentimentSchema = new mongoose.Schema(
  {
    mint: { type: String, unique: true },
    symbol: String,
    score: Number,
    mentions: Number,
    velocity: Number,
    sources: mongoose.Schema.Types.Mixed,
  },
  { timestamps: true },
);

export const User = mongoose.models.User || mongoose.model('User', userSchema);
export const Trade = mongoose.models.Trade || mongoose.model('Trade', tradeSchema);
export const Wallet = mongoose.models.Wallet || mongoose.model('Wallet', walletSchema);
export const CopySub = mongoose.models.CopySub || mongoose.model('CopySub', copySubSchema);
export const AutoRule = mongoose.models.AutoRule || mongoose.model('AutoRule', autoRuleSchema);
export const Sentiment = mongoose.models.Sentiment || mongoose.model('Sentiment', sentimentSchema);
