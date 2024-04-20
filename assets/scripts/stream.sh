yt-dlp --quiet 'https://youtu.be/6TWJaFD6R2s?si=Nj7Sr3w9HFmAeq7l' --ffmpeg-location $(which ffmpeg) -x -o - | ffmpeg -i - -hide_banner -loglevel error -sample_fmt fltp -ar 48000 -f mp3 -
