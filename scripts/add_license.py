import os

old_ts_header = """/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

"""

old_vue_header = """<!--
  - Kerminal - Modern Terminal Emulator & SSH Manager
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  -
  - This program is free software: you can redistribute it and/or modify
  - it under the terms of the GNU General Public License as published by
  - the Free Software Foundation, either version 3 of the License, or
  - (at your option) any later version.
  -
  - This program is distributed in the hope that it will be useful,
  - but WITHOUT ANY WARRANTY; without even the implied warranty of
  - MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  - GNU General Public License for more details.
  -
  - You should have received a copy of the GNU General Public License
  - along with this program.  If not, see <https://www.gnu.org/licenses/>.
-->

"""

short_ts_header = """// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

"""

short_vue_header = """<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

"""

def walk(directory):
    results = []
    for root, dirs, files in os.walk(directory):
        if 'node_modules' in root or 'dist' in root or '.git' in root or 'target' in root:
            continue
        for file in files:
            if file.endswith(('.ts', '.js', '.vue', '.rs')):
                results.append(os.path.join(root, file))
    return results

files = walk('/home/klpod221/Develop/kerminal/src') + walk('/home/klpod221/Develop/kerminal/src-tauri/src')

updated = 0
for file in files:
    with open(file, 'r', encoding='utf-8') as f:
        content = f.read()
    
    modified = False
    
    # Remove old long header if exists
    if content.startswith(old_ts_header):
        content = content[len(old_ts_header):]
        modified = True
    elif content.startswith(old_vue_header):
        content = content[len(old_vue_header):]
        modified = True
        
    # Remove old short header if it was added weirdly (just in case)
    if content.startswith(short_ts_header):
        content = content[len(short_ts_header):]
        modified = True
    elif content.startswith(short_vue_header):
        content = content[len(short_vue_header):]
        modified = True

    # Prepend short header
    if file.endswith('.vue'):
        content = short_vue_header + content
    else:
        content = short_ts_header + content
        
    with open(file, 'w', encoding='utf-8') as f:
        f.write(content)
        
    updated += 1

print(f"Done! Updated header to short SPDX format in {updated} files.")
