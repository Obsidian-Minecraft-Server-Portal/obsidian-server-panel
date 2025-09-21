interface Date
{
    /**
     * Formats the date as a relative string (e.g., "5 minutes ago", "Yesterday", "6 Month's Ago", "in 2 hours") compared to the provided date or the current date if none is provided.
     * @param from
     */
    formatAsRelativeString(from?: Date): string;
}

Date.prototype.formatAsRelativeString = function (from?: Date): string
{
    const now = from || new Date();
    const diff = now.getTime() - this.getTime();
    const seconds = Math.floor(Math.abs(diff) / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);
    const months = Math.floor(days / 30);
    const years = Math.floor(days / 365);

    const isFuture = diff < 0;
    const prefix = isFuture ? "in " : "";
    const suffix = isFuture ? "" : " ago";

    if (seconds < 60)
    {
        return prefix + "just now";
    } else if (minutes < 60)
    {
        return prefix + minutes + " minute" + (minutes === 1 ? "" : "s") + suffix;
    } else if (hours < 24)
    {
        return prefix + hours + " hour" + (hours === 1 ? "" : "s") + suffix;
    } else if (days === 1)
    {
        return isFuture ? "Tomorrow" : "Yesterday";
    } else if (days < 30)
    {
        return prefix + days + " day" + (days === 1 ? "" : "s") + suffix;
    } else if (months < 12)
    {
        return prefix + months + " month" + (months === 1 ? "" : "s") + suffix;
    } else
    {
        return prefix + years + " year" + (years === 1 ? "" : "s") + suffix;
    }
};