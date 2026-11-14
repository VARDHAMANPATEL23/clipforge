export type LayoutMode = "blur" | "crop" | "facecam" | "fit";

export interface Layout {
  mode: LayoutMode;
  zoom: number;
  crop_x: number;
  facecam: [number, number, number, number] | null;
  cam_height: number;
}

export interface Options {
  out: string;
  max_clips: number;
  min_len: number;
  max_len: number;
  target_len: number;
  layout: Layout;
  asr: "whisper" | "none";
  asr_model: "turbo" | "large" | "small";
  language: string | null;
  llm: "none" | "gemini" | "lmstudio" | "ollama" | "openrouter";
  llm_model: string | null;
  vision: boolean;
  niche: string;
  subs: boolean;
  hook: boolean;
  normalize_audio: boolean;
  embed_tag: boolean;
  codec: "h264" | "hevc";
  quality: "high" | "medium" | "low";
  software: boolean;
  workers: number;
  fresh: boolean;
}

export interface Moment {
  start: number;
  end: number;
  score: number;
  peak: number;
  title: string;
  hook: string;
  hashtags: string[];
  reason: string;
}

export interface Clip extends Moment {
  file: string | null;
  caption: string;
}

export interface Progress {
  percent: number;
  downloaded: number;
  total: number | null;
  speed: number | null;
  eta: number | null;
  kind: "video" | "audio";
}

export type JobStatus = "queued" | "running" | "done" | "failed";

export interface Job {
  id: string;
  source: string;
  options: Options;
  status: JobStatus;
  stage: string;
  progress: Progress | null;
  log: string[];
  clips: Clip[];
  out_dir: string | null;
  error: string | null;
  created_at: number;
}

export interface Health {
  ok: boolean;
  version: string;
  apple_silicon: boolean;
  ffmpeg: boolean;
  ffprobe: boolean;
  yt_dlp: boolean;
  libass: boolean;
  encoder: string;
  hardware_encoder: boolean;
  whisper_model: boolean;
}

export interface SettingsView {
  llm: Options["llm"];
  llm_model: string | null;
  base_url: string | null;
  niche: string;
  has_gemini_key: boolean;
  has_openrouter_key: boolean;
}

export interface SettingsUpdate {
  llm?: string;
  llm_model?: string;
  base_url?: string;
  niche?: string;
  gemini_api_key?: string;
  openrouter_api_key?: string;
}

export interface VideoInfo {
  path: string;
  duration: number;
  width: number;
  height: number;
  fps: number;
  has_audio: boolean;
}

export type JobEvent =
  | { type: "snapshot"; job: Job }
  | { type: "stage"; stage: string }
  | { type: "log"; line: string }
  | { type: "progress"; progress: Progress }
  | { type: "status"; status: JobStatus; error?: string };

export interface StorageItem {
  bytes: number;
  files: number;
}

export interface StorageReport {
  downloads: StorageItem;
  analysis: StorageItem;
  uploads: StorageItem;
  models: { name: string; bytes: number }[];
  total: number;
}
