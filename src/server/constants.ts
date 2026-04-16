import path from 'path';
import { isDevelopment } from '@/lib/utils';

const BASE_PATH = process.env.BASE_PATH || (isDevelopment ? process.cwd() : '/');

export const DOWNLOAD_PATH = path.join(BASE_PATH, 'downloads');
export const CACHE_PATH = path.join(BASE_PATH, 'cache');

export const VIDEO_LIST_FILE = 'video-list';
export const COOKIES_FILE = 'cookies';
export const WRITE_TEST_FILE = 'write-test';

export const CACHE_FILE_PREFIX = 'yt-dlp-cache-';
