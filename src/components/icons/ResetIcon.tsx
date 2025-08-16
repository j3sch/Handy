import React from "react";

interface ResetIconProps {
  width?: number;
  height?: number;
  color?: string;
  className?: string;
}

const ResetIcon: React.FC<ResetIconProps> = ({
  width = 20,
  height = 20,
  className = "",
}) => {
  return (
    <svg
      width={width}
      height={height}
      viewBox="0 0 20 20"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <g
        stroke={"currentColor"}
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.5"
      >
        <path d="m13.5 8.5h3v-3" />
        <path d="m13.775 14c-.7863.7419-1.7737 1.2356-2.8389 1.4196-1.06527.1839-2.16109.0498-3.15057-.3855s-1.82875-1.1525-2.41293-2.0621c-.58419-.9095-.88739-1.9711-.87172-3.05197.01567-1.08089.34952-2.13319.95982-3.02543.61031-.89224 1.47001-1.58485 2.4717-1.99129s2.1009-.50868 3.1604-.29396c1.0595.21473 2.0322.7369 2.7965 1.50127l2.6107 2.38938" />
      </g>
    </svg>
  );
};

export default ResetIcon;
