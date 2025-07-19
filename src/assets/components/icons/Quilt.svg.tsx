interface QuiltIconProperties
{
    width?: number | string;
    height?: number | string;
    size?: number | string;
    fill?: string;
}

export default function Quilt(props: QuiltIconProperties)
{
    const {width, height, size, fill} = props;
    return (
        <svg width={width || size || "24"} height={height || size || "24"} xmlnsXlink="http://www.w3.org/1999/xlink" xmlSpace="preserve" fillRule="evenodd" strokeLinecap="round" strokeLinejoin="round" strokeMiterlimit="2" clipRule="evenodd" viewBox="0 0 24 24">
            <defs>
                <path id="quilt" fill={fill || "none"} stroke="currentColor" strokeWidth="65.6" d="M442.5 233.9c0-6.4-5.2-11.6-11.6-11.6h-197c-6.4 0-11.6 5.2-11.6 11.6v197c0 6.4 5.2 11.6 11.6 11.6h197c6.4 0 11.6-5.2 11.6-11.7v-197Z"></path>
            </defs>
            <path fill={fill || "none"} d="M0 0h24v24H0z"></path>
            <use xlinkHref="#quilt" strokeWidth="65.6" transform="matrix(.03053 0 0 .03046 -3.2 -3.2)"></use>
            <use xlinkHref="#quilt" strokeWidth="65.6" transform="matrix(.03053 0 0 .03046 -3.2 7)"></use>
            <use xlinkHref="#quilt" strokeWidth="65.6" transform="matrix(.03053 0 0 .03046 6.9 -3.2)"></use>
            <path fill={fill || "none"} stroke="currentColor" strokeWidth="70.4" d="M442.5 234.8c0-7-5.6-12.5-12.5-12.5H234.7c-6.8 0-12.4 5.6-12.4 12.5V430c0 6.9 5.6 12.5 12.4 12.5H430c6.9 0 12.5-5.6 12.5-12.5V234.8Z" transform="rotate(45 3.5 24) scale(.02843 .02835)"></path>
        </svg>
    );
}