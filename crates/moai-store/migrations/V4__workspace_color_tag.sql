-- V4: workspace color tags (D-5 feature)
--
-- Adds color_tag column for visual workspace identification.
-- NULL means no color assigned (default).

ALTER TABLE workspaces ADD COLUMN color_tag TEXT;
