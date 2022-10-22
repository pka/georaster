all: N265E425.tif

# EU-DEM tile for testing

N265E425.zip:
	wget http://www.openmaps.online/eudem_download/N265E425.zip

N265E425/N265E425.tif: N265E425.zip
	unzip $<
	touch $@

N265E425.tif: N265E425/N265E425.tif
	gdal_translate $< $@ -of COG -co COMPRESS=Deflate -ot UInt16
	# gdalinfo N265E425.tif 
	#
	# Driver: GTiff/GeoTIFF
	# Files: N265E425.tif
	# Size is 5000, 5000
	# Coordinate System is:
	# PROJCRS["ETRS89_ETRS_LAEA",
	#     ...
	# Data axis to CRS axis mapping: 1,2
	# Origin = (4250000.000000000000000,2700000.000000000000000)
	# Pixel Size = (10.000000000000000,-10.000000000000000)
	# Metadata:
	#   AREA_OR_POINT=Area
	# Image Structure Metadata:
	#   COMPRESSION=DEFLATE
	#   INTERLEAVE=BAND
	#   LAYOUT=COG
	# Corner Coordinates:
	# Upper Left  ( 4250000.000, 2700000.000) (  9d 3'35.67"E, 47d24'34.38"N)
	# Lower Left  ( 4250000.000, 2650000.000) (  9d 4' 4.68"E, 46d57'34.17"N)
	# Upper Right ( 4300000.000, 2700000.000) (  9d43'18.96"E, 47d24'47.70"N)
	# Lower Right ( 4300000.000, 2650000.000) (  9d43'27.54"E, 46d57'47.36"N)
	# Center      ( 4275000.000, 2675000.000) (  9d23'36.73"E, 47d11'12.75"N)
	# Band 1 Block=512x512 Type=UInt16, ColorInterp=Gray
	#   NoData Value=0
	#   Overviews: 2500x2500, 1250x1250, 625x625, 312x312

N265E425.png: N265E425.tif
	gdal_translate $< $@ -co ZLEVEL=1
