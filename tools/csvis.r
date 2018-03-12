args <- commandArgs(trailingOnly=T) 
fname <- args[1]

if (is.na(fname)) {
    stop("Usage ./csvis.r ~.csv") 
}

f <- read.csv(fname, header = T)
plot(f)
