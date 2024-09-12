export function formatMessageTime(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const localDate = new Date(date.toLocaleString('en-US', { timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone }));
    
    const isToday = localDate.toDateString() === now.toDateString();
    const isThisWeek = now.getTime() - localDate.getTime() < 7 * 24 * 60 * 60 * 1000;
    const isThisYear = localDate.getFullYear() === now.getFullYear();
  
    if (isToday) {
      return localDate.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
    } else if (isThisWeek) {
      return localDate.toLocaleDateString('en-US', { weekday: 'short' });
    } else if (isThisYear) {
      return localDate.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    } else {
      return localDate.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
    }
  }