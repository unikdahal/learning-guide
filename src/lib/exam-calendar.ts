const DAY = 86_400_000;
export function validExamDate(value: unknown): value is string {
  if(value === '') return true;
  if(typeof value !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const [year,month,day]=value.split('-').map(Number);
  if(year! < 2000 || year! > 2100) return false;
  const date = new Date(Date.UTC(year!,month!-1,day!));
  return date.getUTCFullYear()===year && date.getUTCMonth()===month!-1 && date.getUTCDate()===day;
}
export function daysUntil(date: string, now = new Date()): number | null {
  if(!date || !validExamDate(date)) return null;
  const [year,month,day]=date.split('-').map(Number);
  return Math.round((Date.UTC(year!,month!-1,day!) - Date.UTC(now.getFullYear(),now.getMonth(),now.getDate())) / DAY);
}
