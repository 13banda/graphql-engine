export const getPathRoot = (path: string | undefined | null = '') =>
  path?.split('/')[1] ?? '';

export const stripTrailingSlash = (url: string) => {
  if (url && url.endsWith('/')) {
    return url.slice(0, -1);
  }

  return url;
};
